mod alignment;

pub use alignment::*;
use {
    crate::generic::{default, panic},
    core::fmt,
    expect_test::expect,
    kstring::KString,
    std::{
        cmp::Ordering,
        collections::BTreeMap,
        fmt::{Debug, Display},
        hash::{Hash, Hasher},
        io::{self, Read, Seek, SeekFrom, Write},
        iter::once,
        ops::{Add, AddAssign, Deref, DerefMut, Index, Range},
    },
    tracing::warn,
};

/// In-memory output buffer supporting multiple [overlapping hierarchical markup
/// tracks][1].
///
/// Can be concatenated with other `OutputBuffer`s while preserving tags.
///
/// [1]: https://en.wikipedia.org/wiki/Overlapping_markup
#[derive(Debug, Default, Clone)]
pub struct OutputBuffer {
    /// the actual data
    bytes: Vec<u8>,
    /// tracks of complete tags
    tag_tracks: BTreeMap<KString, Vec<TaggedRange>>,
    /// tracks of stacks of incomplete tags
    tag_stacks: BTreeMap<KString, Vec<TaggedRange>>,
}

impl OutputBuffer {
    pub fn new() -> Self {
        default()
    }

    pub fn without_tag(data: impl AsRef<[u8]>) -> Self {
        Self {
            bytes: data.as_ref().to_vec(),
            tag_tracks: default(),
            tag_stacks: default(),
        }
    }

    pub fn with_tag(
        data: impl AsRef<[u8]>,
        track: impl Into<KString>,
        tag: impl Into<KString>,
    ) -> Self {
        let mut buffer = Self::new();
        *buffer.tagged(track, tag) += data.as_ref();
        buffer
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub fn tagged(
        &mut self,
        track: impl Into<KString>,
        tag: impl Into<KString>,
    ) -> InOutputBufferTag<'_> {
        let track = track.into();
        let tag = tag.into();
        self.start(track.clone(), tag.clone());
        InOutputBufferTag {
            buffer: Some(self),
            track,
            tag,
        }
    }

    pub fn push(&mut self, byte: u8) {
        self.bytes.push(byte);
    }

    pub fn extend<Data>(&mut self, data: Data)
    where OutputBuffer: AddAssign<Data> {
        *self += data;
    }

    /// Concatenates the contents of other onto self.
    /// Closed tags are copied over, nested under the current tag if one is open
    /// on the corresponding track, and with their offsets adjusted
    /// appropriately. Unclosed tags are silently discarded.
    pub fn concat(&mut self, other: &Self) {
        let offset = self.bytes.len();
        self.bytes.extend_from_slice(&other.bytes);

        for (other_track, other_tags) in &other.tag_tracks {
            let track = self.tag_tracks.entry(other_track.clone()).or_default();
            let stack = self.tag_stacks.entry(other_track.clone()).or_default();
            let depth = stack.len();
            let parent = stack.last_mut().map(|t| &mut t.children).unwrap_or(track);

            for tag in other_tags {
                parent.push(tag.add(offset, depth));
            }
        }
    }

    /// Pushes a tag onto the stack of tags.
    pub fn start(&mut self, track: impl Into<KString>, tag: impl Into<KString>) {
        let track = track.into();
        let tag = tag.into();

        let start = self.offset();
        let stack = self.tag_stacks.entry(track).or_default();
        let depth = stack.len();
        stack.push(TaggedRange::new(start, depth, tag));
    }

    /// Pops a tag off the stack of tags, finalizes it at the current index, and
    /// then adds it as a child of the next tag on the stack, if one exists,
    /// or else adds it to the list of complete tags.
    pub fn end(&mut self, track: impl Into<KString>, tag: impl Into<KString>) {
        let track = track.into();
        let _tag = tag.into();

        let end = self.offset();
        let stack = self.tag_stacks.get_mut(&track).unwrap();
        let mut range = stack.pop().unwrap();
        range.end = end;
        if let Some(parent) = stack.last_mut() {
            parent.children.push(range);
        } else {
            self.tag_tracks.entry(track).or_default().push(range);
        }
    }

    pub fn offset(&self) -> usize {
        self.bytes.len()
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn tracks(&self) -> impl Iterator<Item = &KString> {
        self.tag_tracks.keys()
    }

    pub fn walk_tags<'a>(
        &'a self,
        track: impl Into<KString>,
    ) -> Box<dyn 'a + Iterator<Item = OutputBufferWalkEntry>> {
        let track = track.into();
        let mut tags: Vec<OutputBufferWalkEntry> = self
            .tag_tracks
            .get(&track)
            .iter()
            .flat_map(|t| t.iter())
            .flat_map(|t| t.iter())
            // .filter(|t| t.start != t.end)
            .flat_map(|t| {
                [
                    OutputBufferWalkEntry {
                        index: t.start,
                        length: t.len(),
                        depth: t.depth,
                        is_closing: false,
                        track: track.clone(),
                        tag: t.name.clone(),
                    },
                    OutputBufferWalkEntry {
                        index: t.end,
                        length: t.len(),
                        depth: t.depth,
                        is_closing: true,
                        track: track.clone(),
                        tag: t.name.clone(),
                    },
                ]
            })
            .collect();
        tags.sort();
        Box::new(tags.into_iter())
    }

    pub fn root_tags(&self, track: impl Into<KString>) -> Option<&[TaggedRange]> {
        let track = track.into();

        if !self.tag_stacks.values().all(|stack| stack.is_empty()) {
            warn!("tag track {track:?} requested but it still has incomplete tags in its stack.");
        }
        self.tag_tracks
            .get(&track)
            .map(|v| v.as_slice())
            .filter(|v| !v.is_empty())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputBufferWalkEntry {
    index: usize,
    length: usize,
    depth: usize,
    is_closing: bool,
    track: KString,
    tag: KString,
}

impl PartialOrd for OutputBufferWalkEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OutputBufferWalkEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Sort by index primarily, of course.
        let by_index = self.index.cmp(&other.index);
        if by_index != Ordering::Equal {
            return by_index;
        }

        // Closing tags go before opening tags at the same index.
        let by_closing = self.is_closing.cmp(&other.is_closing).reverse();
        if by_closing != Ordering::Equal {
            return by_closing;
        }

        // For opening tags, we want least depth first, but for closing tags,
        // we want greatest depth first.
        let by_depth = if self.is_closing {
            self.depth.cmp(&other.depth).reverse()
        } else {
            self.depth.cmp(&other.depth)
        };
        if by_depth != Ordering::Equal {
            return by_depth;
        }

        (&self.track, &self.tag).cmp(&(&other.track, &other.tag))
    }
}

impl Display for OutputBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn write_bytes(f: &mut fmt::Formatter, bytes: &[u8], indentation: &str) -> fmt::Result {
            let text_wrap_at = 96;
            let mut line_text_length = 0;

            f.write_str(indentation)?;

            for window in once(b' ')
                .chain(bytes.iter().copied())
                .chain(once(b' '))
                .collect::<Vec<_>>()
                .windows(3)
            {
                let (previous, byte, next) = (window[0], window[1], window[2]);

                let is_single_space_inline = byte == b' ' && previous != b' ' && next != b' ';

                let mut wrap_modifier = 0;
                if !byte.is_ascii_graphic() && next.is_ascii_graphic() {
                    wrap_modifier += 32;
                }

                let mut to_write;

                if byte.is_ascii_graphic() && !matches!(byte, b'<' | b'>' | b'&') {
                    to_write = format!("{}", byte as char);
                    line_text_length += 1;
                } else {
                    to_write = format!("&#x{byte:02X};");
                    line_text_length += 6;
                }

                // XXX: we shouldn't be wrapping before the end of string, or before an
                // unescaped space
                if line_text_length + wrap_modifier >= text_wrap_at {
                    to_write += "\n";
                    to_write += indentation;
                    line_text_length = 0;
                }

                if to_write == "&#x20;" && is_single_space_inline {
                    to_write = " ".to_string();
                }

                write!(f, "{to_write}")?;
            }

            Ok(())
        }

        let mut tags = Vec::new();
        for track in self.tracks() {
            tags.extend(self.walk_tags(track.clone()));
        }
        tags.sort();

        let mut index = 0;

        let mut next_depth = 0;

        for tag in tags {
            let before = &self.bytes[index..tag.index];
            if !before.is_empty() {
                write_bytes(
                    f,
                    before,
                    &format!("{:indent$}", "", indent = (next_depth) * 2),
                )?;
                index = tag.index;
                writeln!(f)?;
            }

            write!(f, "{:indent$}", "", indent = (tag.depth) * 2)?;

            if !tag.is_closing {
                if tag.track != tag.tag {
                    write!(
                        f,
                        "<{}-{} offset=\"{}\" length=\"{}\">",
                        tag.track, tag.tag, tag.index, tag.length
                    )?;
                } else {
                    write!(
                        f,
                        "<{} offset=\"{}\" length=\"{}\">",
                        tag.track, tag.index, tag.length
                    )?;
                }
                next_depth = tag.depth + 1;
            } else {
                if tag.track != tag.tag {
                    write!(f, "</{}-{}>", tag.track, tag.tag)?;
                } else {
                    write!(f, "</{}>", tag.track)?;
                }
                next_depth = tag.depth;
            }
            writeln!(f)?;
        }
        write_bytes(f, &self.bytes[index..], "")?;

        Ok(())
    }
}

impl AddAssign<OutputBuffer> for OutputBuffer {
    fn add_assign(&mut self, other: OutputBuffer) {
        self.concat(&other);
    }
}

impl AddAssign<&OutputBuffer> for OutputBuffer {
    fn add_assign(&mut self, other: &OutputBuffer) {
        self.concat(other);
    }
}

impl AddAssign<&[u8]> for OutputBuffer {
    fn add_assign(&mut self, other: &[u8]) {
        self.bytes.extend_from_slice(other);
    }
}

impl<const N: usize> AddAssign<&[u8; N]> for OutputBuffer {
    fn add_assign(&mut self, other: &[u8; N]) {
        self.bytes.extend_from_slice(other.as_slice());
    }
}

impl AddAssign<OutputBuffer> for &mut OutputBuffer {
    fn add_assign(&mut self, other: OutputBuffer) {
        self.concat(&other);
    }
}

impl AddAssign<&OutputBuffer> for &mut OutputBuffer {
    fn add_assign(&mut self, other: &OutputBuffer) {
        self.concat(other);
    }
}

impl AddAssign<&[u8]> for &mut OutputBuffer {
    fn add_assign(&mut self, other: &[u8]) {
        self.bytes.extend_from_slice(other);
    }
}

impl<const N: usize> AddAssign<&[u8; N]> for &mut OutputBuffer {
    fn add_assign(&mut self, other: &[u8; N]) {
        self.bytes.extend_from_slice(other.as_slice());
    }
}

impl IntoIterator for OutputBuffer {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.bytes.into_iter()
    }
}

impl From<OutputBuffer> for Vec<u8> {
    fn from(buffer: OutputBuffer) -> Self {
        buffer.into_bytes()
    }
}

impl AsRef<[u8]> for OutputBuffer {
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

impl AsRef<Vec<u8>> for OutputBuffer {
    fn as_ref(&self) -> &Vec<u8> {
        &self.bytes
    }
}

impl AsMut<[u8]> for OutputBuffer {
    fn as_mut(&mut self) -> &mut [u8] {
        self.bytes.as_mut_slice()
    }
}

impl AsMut<Vec<u8>> for OutputBuffer {
    fn as_mut(&mut self) -> &mut Vec<u8> {
        &mut self.bytes
    }
}

impl Offset for OutputBuffer {
    fn offset(&mut self) -> usize {
        OutputBuffer::offset(self)
    }

    fn len(&mut self) -> usize {
        OutputBuffer::len(self)
    }
}

impl Write for OutputBuffer {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.bytes.write(buf)
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        self.bytes.flush()
    }
}

impl<'a> Write for InOutputBufferTag<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.buffer.as_mut().unwrap().write(buf)
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        self.buffer.as_mut().unwrap().flush()
    }
}

impl<'a> Offset for InOutputBufferTag<'a> {
    fn offset(&mut self) -> usize {
        self.buffer.as_mut().unwrap().offset()
    }

    fn len(&mut self) -> usize {
        self.buffer.as_mut().unwrap().len()
    }
}

impl<'a> AddAssign<OutputBuffer> for InOutputBufferTag<'a> {
    fn add_assign(&mut self, other: OutputBuffer) {
        self.buffer.as_mut().unwrap().concat(&other);
    }
}

impl<'a> AddAssign<&OutputBuffer> for InOutputBufferTag<'a> {
    fn add_assign(&mut self, other: &OutputBuffer) {
        self.buffer.as_mut().unwrap().concat(other);
    }
}

impl<'a> AddAssign<&[u8]> for InOutputBufferTag<'a> {
    fn add_assign(&mut self, other: &[u8]) {
        *self.buffer.as_mut().unwrap() += other;
    }
}

impl<'a> AddAssign<OutputBuffer> for &mut InOutputBufferTag<'a> {
    fn add_assign(&mut self, other: OutputBuffer) {
        self.buffer.as_mut().unwrap().concat(&other);
    }
}

impl<'a> AddAssign<&OutputBuffer> for &mut InOutputBufferTag<'a> {
    fn add_assign(&mut self, other: &OutputBuffer) {
        self.buffer.as_mut().unwrap().concat(other);
    }
}

impl<'a> AddAssign<&[u8]> for &mut InOutputBufferTag<'a> {
    fn add_assign(&mut self, other: &[u8]) {
        *self.buffer.as_mut().unwrap() += other;
    }
}

impl<'a, const N: usize> AddAssign<&[u8; N]> for &mut InOutputBufferTag<'a> {
    fn add_assign(&mut self, other: &[u8; N]) {
        *self.buffer.as_mut().unwrap() += other;
    }
}

impl<'a, const N: usize> AddAssign<&[u8; N]> for InOutputBufferTag<'a> {
    fn add_assign(&mut self, other: &[u8; N]) {
        *self.buffer.as_mut().unwrap() += other;
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
pub struct TaggedRange {
    /// The first index of the range (inclusive).
    start: usize,
    /// The last index of the range (exclusive).
    end: usize,
    /// How many ancestors this range has, or 0 if it's a root.
    depth: usize,
    /// The name/tag of the region.
    name: KString,
    /// Ad-hoc attributes
    attributes: Vec<(KString, KString)>,
    /// The child sub-regions of this region.
    children: Vec<TaggedRange>,
}

impl TaggedRange {
    fn new(start: usize, depth: usize, name: impl Into<KString>) -> Self {
        Self {
            start,
            depth,
            end: usize::MAX,
            name: name.into(),
            children: Vec::new(),
            attributes: Vec::new(),
        }
    }

    fn finalize(mut self, end: usize) -> Self {
        assert_eq!(
            self.end,
            usize::MAX,
            "attempted to finalize a TaggedRange twice"
        );
        if end == self.start {
            warn!("finalizing an empty TaggedRange");
        }
        self.end = end;
        self
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn name(&self) -> &KString {
        &self.name
    }

    pub fn children(&self) -> &[TaggedRange] {
        &self.children
    }

    pub fn attributes(&self) -> &[(KString, KString)] {
        &self.attributes
    }

    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }

    pub fn iter<'a>(&'a self) -> Box<dyn 'a + Iterator<Item = &'a TaggedRange>> {
        // XXX: wtf are you doing fix this
        let mut items = vec![self];
        for child in &self.children {
            items.extend(child.iter());
        }
        Box::new(items.into_iter())
    }

    fn add(&self, offset: usize, depth: usize) -> Self {
        Self {
            start: self.start + offset,
            end: self.end + offset,
            attributes: self.attributes.clone(),
            name: self.name.clone(),
            depth: self.depth + depth,
            children: self.children.iter().map(|c| c.add(offset, depth)).collect(),
        }
    }
}

impl Index<TaggedRange> for [u8] {
    type Output = [u8];

    fn index(&self, index: TaggedRange) -> &Self::Output {
        &self[index.start..=index.end]
    }
}

impl Index<TaggedRange> for OutputBuffer {
    type Output = [u8];

    fn index(&self, index: TaggedRange) -> &Self::Output {
        &self.bytes[index.start..=index.end]
    }
}

#[derive(Debug)]
pub struct InOutputBufferTag<'buffer> {
    buffer: Option<&'buffer mut OutputBuffer>,
    track: KString,
    tag: KString,
}

impl<'buffer> InOutputBufferTag<'buffer> {
    pub fn closed(mut self) -> &'buffer mut OutputBuffer {
        self.buffer.take().unwrap()
    }
}

impl Deref for InOutputBufferTag<'_> {
    type Target = OutputBuffer;

    fn deref(&self) -> &Self::Target {
        self.buffer.as_ref().unwrap()
    }
}

impl DerefMut for InOutputBufferTag<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buffer.as_mut().unwrap()
    }
}

impl Drop for InOutputBufferTag<'_> {
    fn drop(&mut self) {
        if let Some(buffer) = &mut self.buffer {
            buffer.end(self.track.clone(), self.tag.clone());
        }
    }
}

pub fn output_buffer() -> OutputBuffer {
    default()
}

pub trait Offset {
    fn offset(&mut self) -> usize;

    fn len(&mut self) -> usize;
}

impl<T> Offset for T
where T: Seek
{
    fn offset(&mut self) -> usize {
        let Ok(position) = self.stream_position() else {
            return usize::MAX;
        };
        let Ok(position) = usize::try_from(position) else {
            return usize::MAX;
        };
        position
    }

    fn len(&mut self) -> usize {
        let old_pos = self.stream_position().unwrap_or(u64::MAX);
        let len = self.seek(SeekFrom::End(0)).unwrap_or(u64::MAX);
        if old_pos != len {
            self.seek(SeekFrom::Start(old_pos)).ok();
        }
        len.try_into().unwrap_or(usize::MAX)
    }
}

#[cfg(test)]
#[test]
fn test_output_buffer() -> Result<(), panic> {
    crate::dev::init!();

    let mut buffer = output_buffer();

    {
        let mut buffer = buffer.tagged("png", "png");

        buffer.start("png", "signature");
        buffer.extend(b"\x89PNG\r");
        buffer.end("png", "signature");

        let mut buffer = buffer.tagged("png", "header");

        buffer.extend(b"\x00\x00\x00\rIHDR");

        let mut buffer = buffer.closed();

        buffer.start("png", "body");

        buffer.start("ZIP", "ZIP");

        let mut sub = output_buffer();
        sub.tagged("png", "sub").extend(b"\x90PNG\r");
        buffer += sub;

        buffer.end("png", "body");

        buffer.end("ZIP", "ZIP");
        buffer.end("png", "PNG");
    }

    expect![[r#"
        Some(
            [
                TaggedRange {
                    start: 17,
                    end: 22,
                    depth: 0,
                    name: "ZIP",
                    children: [],
                },
            ],
        )
    "#]]
    .assert_debug_eq(&buffer.root_tags("ZIP"));

    expect![[r#"
        Some(
            [
                TaggedRange {
                    start: 0,
                    end: 22,
                    depth: 0,
                    name: "PNG",
                    children: [
                        TaggedRange {
                            start: 5,
                            end: 22,
                            depth: 1,
                            name: "IHDR",
                            children: [
                                TaggedRange {
                                    start: 17,
                                    end: 17,
                                    depth: 2,
                                    name: "signature",
                                    children: [],
                                },
                                TaggedRange {
                                    start: 17,
                                    end: 22,
                                    depth: 2,
                                    name: "image data",
                                    children: [
                                        TaggedRange {
                                            start: 17,
                                            end: 22,
                                            depth: 3,
                                            name: "sub",
                                            children: [],
                                        },
                                    ],
                                },
                            ],
                        },
                    ],
                },
            ],
        )
    "#]]
    .assert_debug_eq(&buffer.root_tags("PNG"));

    expect![[r#"
        <PNG:PNG>
            &#x89;PNG&#x0D;
            <PNG:IHDR>
                &#0;&#0;&#0;&#x0D;IHDRtest
                </PNG:signature>
        <ZIP:ZIP>
                <PNG:image data>
                <PNG:signature>
                    <PNG:sub>
                            &#x90;PNG&#x0D;
                    </PNG:sub>
                </PNG:image data>
            </PNG:IHDR>
        </PNG:PNG>
        </ZIP:ZIP>
    "#]]
    .assert_eq(&buffer.to_string());

    Ok(())
}
