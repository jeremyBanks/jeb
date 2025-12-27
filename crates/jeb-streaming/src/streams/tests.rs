#[cfg(test)]
mod tests {
    use crate::{streams::*, Item};
    use futures::StreamExt;

    // ===== Transform Tests: split_after() =====

    #[tokio::test]
    async fn test_split_after_custom_delimiter() {
        let source = text_source(vec!["hello||world||".to_string()]);
        let result: Vec<_> = split_after(source, "||")
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("hello||".into()));
        assert_eq!(result[1], Item::Text("world||".into()));
    }

    #[tokio::test]
    async fn test_split_after_multi_char_pattern() {
        let source = text_source(vec!["data<SEP>more<SEP>end".to_string()]);
        let result: Vec<_> = split_after(source, "<SEP>")
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Item::Text("data<SEP>".into()));
        assert_eq!(result[1], Item::Text("more<SEP>".into()));
        assert_eq!(result[2], Item::Text("end".into()));
    }

    #[tokio::test]
    async fn test_split_after_pattern_split_across_chunks() {
        let source = text_source(vec![
            "hello<".to_string(),
            "SEP>world<SE".to_string(),
            "P>end".to_string(),
        ]);
        let result: Vec<_> = split_after(source, "<SEP>")
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Item::Text("hello<SEP>".into()));
        assert_eq!(result[1], Item::Text("world<SEP>".into()));
        assert_eq!(result[2], Item::Text("end".into()));
    }

    #[tokio::test]
    async fn test_split_after_bytes_custom_pattern() {
        let source = bytes_source(vec![b"data||more||".to_vec()]);
        let result: Vec<_> = split_after(source, "||")
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Bytes(b"data||".to_vec().into()));
        assert_eq!(result[1], Item::Bytes(b"more||".to_vec().into()));
    }

    #[tokio::test]
    async fn test_split_after_no_match() {
        let source = text_source(vec!["hello world".to_string()]);
        let result: Vec<_> = split_after(source, "||")
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Text("hello world".into()));
    }

    #[tokio::test]
    async fn test_split_after_empty_pattern_flush() {
        let source = text_source(vec!["data||partial".to_string()]);
        let result: Vec<_> = split_after(source, "||")
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("data||".into()));
        assert_eq!(result[1], Item::Text("partial".into()));
    }

    // ===== Transform Tests: lines() =====

    #[tokio::test]
    async fn test_lines_text_single_line() {
        let source = text_source(vec!["hello\n".to_string()]);
        let result: Vec<_> = lines(source)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Text("hello\n".into()));
    }

    #[tokio::test]
    async fn test_lines_text_multiple_lines() {
        let source = text_source(vec!["hello\nworld\n".to_string()]);
        let result: Vec<_> = lines(source)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("hello\n".into()));
        assert_eq!(result[1], Item::Text("world\n".into()));
    }

    #[tokio::test]
    async fn test_lines_text_partial_line() {
        let source = text_source(vec!["hello\nworld".to_string()]);
        let result: Vec<_> = lines(source)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("hello\n".into()));
        assert_eq!(result[1], Item::Text("world".into()));
    }

    #[tokio::test]
    async fn test_lines_text_split_across_chunks() {
        let source = text_source(vec![
            "hel".to_string(),
            "lo\nwor".to_string(),
            "ld\n".to_string(),
        ]);
        let result: Vec<_> = lines(source)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("hello\n".into()));
        assert_eq!(result[1], Item::Text("world\n".into()));
    }

    #[tokio::test]
    async fn test_lines_bytes_single_line() {
        let source = bytes_source(vec![b"hello\n".to_vec()]);
        let result: Vec<_> = lines(source)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Bytes(b"hello\n".to_vec().into()));
    }

    #[tokio::test]
    async fn test_lines_bytes_multiple_lines() {
        let source = bytes_source(vec![b"hello\nworld\n".to_vec()]);
        let result: Vec<_> = lines(source)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Bytes(b"hello\n".to_vec().into()));
        assert_eq!(result[1], Item::Bytes(b"world\n".to_vec().into()));
    }

    #[tokio::test]
    async fn test_lines_bytes_partial_line() {
        let source = bytes_source(vec![b"hello\nworld".to_vec()]);
        let result: Vec<_> = lines(source)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Bytes(b"hello\n".to_vec().into()));
        assert_eq!(result[1], Item::Bytes(b"world".to_vec().into()));
    }

    #[tokio::test]
    async fn test_lines_bytes_split_across_chunks() {
        let source = bytes_source(vec![
            b"hel".to_vec(),
            b"lo\nwor".to_vec(),
            b"ld\n".to_vec(),
        ]);
        let result: Vec<_> = lines(source)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Bytes(b"hello\n".to_vec().into()));
        assert_eq!(result[1], Item::Bytes(b"world\n".to_vec().into()));
    }

    #[tokio::test]
    async fn test_lines_empty_lines() {
        let source = text_source(vec!["\n\n\n".to_string()]);
        let result: Vec<_> = lines(source)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Item::Text("\n".into()));
        assert_eq!(result[1], Item::Text("\n".into()));
        assert_eq!(result[2], Item::Text("\n".into()));
    }

    #[tokio::test]
    async fn test_lines_no_newline() {
        let source = text_source(vec!["hello world".to_string()]);
        let result: Vec<_> = lines(source)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Text("hello world".into()));
    }

    // ===== Transform Tests: chunks() =====

    #[tokio::test]
    async fn test_chunks_text_exact_size() {
        let source = text_source(vec!["12345".to_string()]);
        let result: Vec<_> = chunks(source, 5)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Text("12345".into()));
    }

    #[tokio::test]
    async fn test_chunks_text_multiple_chunks() {
        let source = text_source(vec!["1234567890".to_string()]);
        let result: Vec<_> = chunks(source, 3)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], Item::Text("123".into()));
        assert_eq!(result[1], Item::Text("456".into()));
        assert_eq!(result[2], Item::Text("789".into()));
        assert_eq!(result[3], Item::Text("0".into()));
    }

    #[tokio::test]
    async fn test_chunks_text_partial_chunk() {
        let source = text_source(vec!["12345".to_string()]);
        let result: Vec<_> = chunks(source, 3)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("123".into()));
        assert_eq!(result[1], Item::Text("45".into()));
    }

    #[tokio::test]
    async fn test_chunks_text_across_stream_items() {
        let source = text_source(vec!["123".to_string(), "456".to_string(), "789".to_string()]);
        let result: Vec<_> = chunks(source, 5)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("12345".into()));
        assert_eq!(result[1], Item::Text("6789".into()));
    }

    #[tokio::test]
    async fn test_chunks_bytes_exact_size() {
        let source = bytes_source(vec![b"12345".to_vec()]);
        let result: Vec<_> = chunks(source, 5)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Bytes(b"12345".to_vec().into()));
    }

    #[tokio::test]
    async fn test_chunks_bytes_multiple_chunks() {
        let source = bytes_source(vec![b"1234567890".to_vec()]);
        let result: Vec<_> = chunks(source, 3)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], Item::Bytes(b"123".to_vec().into()));
        assert_eq!(result[1], Item::Bytes(b"456".to_vec().into()));
        assert_eq!(result[2], Item::Bytes(b"789".to_vec().into()));
        assert_eq!(result[3], Item::Bytes(b"0".to_vec().into()));
    }

    #[tokio::test]
    async fn test_chunks_bytes_partial_chunk() {
        let source = bytes_source(vec![b"12345".to_vec()]);
        let result: Vec<_> = chunks(source, 3)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Bytes(b"123".to_vec().into()));
        assert_eq!(result[1], Item::Bytes(b"45".to_vec().into()));
    }

    #[tokio::test]
    async fn test_chunks_zero_defaults_to_65536() {
        let large_text = "a".repeat(70000);
        let source = text_source(vec![large_text.clone()]);
        let result: Vec<_> = chunks(source, 0)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 2);
        match &result[0] {
            Item::Text(s) => assert_eq!(s.chars().count(), 65536),
            _ => panic!("Expected text item"),
        }
        match &result[1] {
            Item::Text(s) => assert_eq!(s.chars().count(), 70000 - 65536),
            _ => panic!("Expected text item"),
        }
    }

    #[tokio::test]
    async fn test_chunks_text_unicode() {
        // Test that chunks work correctly with multi-byte UTF-8 characters
        let source = text_source(vec!["🦀🦀🦀🦀🦀".to_string()]);
        let result: Vec<_> = chunks(source, 3)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("🦀🦀🦀".into()));
        assert_eq!(result[1], Item::Text("🦀🦀".into()));
    }

    // ===== Source Tests =====

    #[tokio::test]
    async fn test_text_source_single_item() {
        let result: Vec<_> = text_source(vec!["hello".to_string()])
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Text("hello".into()));
    }

    #[tokio::test]
    async fn test_text_source_multiple_items() {
        let result: Vec<_> = text_source(vec!["hello".to_string(), "world".to_string()])
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("hello".into()));
        assert_eq!(result[1], Item::Text("world".into()));
    }

    #[tokio::test]
    async fn test_text_source_empty() {
        let result: Vec<_> = text_source(Vec::<String>::new())
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_bytes_source_single_item() {
        let result: Vec<_> = bytes_source(vec![b"hello".to_vec()])
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Bytes(b"hello".to_vec().into()));
    }

    #[tokio::test]
    async fn test_bytes_source_multiple_items() {
        let result: Vec<_> = bytes_source(vec![b"hello".to_vec(), b"world".to_vec()])
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Bytes(b"hello".to_vec().into()));
        assert_eq!(result[1], Item::Bytes(b"world".to_vec().into()));
    }

    #[tokio::test]
    async fn test_bytes_source_empty() {
        let result: Vec<_> = bytes_source(Vec::<Vec<u8>>::new())
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 0);
    }

    // ===== Mixed Type Tests =====

    #[tokio::test]
    async fn test_lines_switches_between_text_and_bytes() {
        use async_stream::stream;

        // Create a stream that mixes text and bytes
        let mixed_source = stream! {
            yield Ok(Item::Text("hello\n".into()));
            yield Ok(Item::Bytes(b"world\n".to_vec().into()));
            yield Ok(Item::Text("foo\n".into()));
        };

        let result: Vec<_> = lines(mixed_source)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Item::Text("hello\n".into()));
        assert_eq!(result[1], Item::Bytes(b"world\n".to_vec().into()));
        assert_eq!(result[2], Item::Text("foo\n".into()));
    }

    #[tokio::test]
    async fn test_chunks_switches_between_text_and_bytes() {
        use async_stream::stream;

        // Create a stream that mixes text and bytes
        let mixed_source = stream! {
            yield Ok(Item::Text("12345".into()));
            yield Ok(Item::Bytes(b"67890".to_vec().into()));
        };

        let result: Vec<_> = chunks(mixed_source, 3)
            .map(|r| r.unwrap())
            .collect()
            .await;

        // Should flush text buffer when switching to bytes
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], Item::Text("123".into()));
        assert_eq!(result[1], Item::Text("45".into()));
        assert_eq!(result[2], Item::Bytes(b"678".to_vec().into()));
        assert_eq!(result[3], Item::Bytes(b"90".to_vec().into()));
    }

    // ===== Error Handling Tests =====

    #[tokio::test]
    async fn test_lines_propagates_errors() {
        use async_stream::stream;

        let error_source = stream! {
            yield Ok(Item::Text("hello\n".into()));
            yield Err("test error");
            yield Ok(Item::Text("world\n".into()));
        };

        let result: Vec<_> = lines(error_source).collect().await;

        assert_eq!(result.len(), 3);
        assert!(result[0].is_ok());
        assert_eq!(result[1], Err("test error"));
        assert!(result[2].is_ok());
    }

    #[tokio::test]
    async fn test_chunks_propagates_errors() {
        use async_stream::stream;

        let error_source = stream! {
            yield Ok(Item::Text("12345".into()));
            yield Err("test error");
            yield Ok(Item::Text("67890".into()));
        };

        let result: Vec<_> = chunks(error_source, 3).collect().await;

        assert!(result.iter().any(|r| r.is_err()));
    }
}
