#[cfg(test)]
mod tests {
    use {
        crate::{
            Item,
            streams::*,
        },
        futures::StreamExt,
    };

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
        let result: Vec<_> = lines(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Text("hello\n".into()));
    }

    #[tokio::test]
    async fn test_lines_text_multiple_lines() {
        let source = text_source(vec!["hello\nworld\n".to_string()]);
        let result: Vec<_> = lines(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("hello\n".into()));
        assert_eq!(result[1], Item::Text("world\n".into()));
    }

    #[tokio::test]
    async fn test_lines_text_partial_line() {
        let source = text_source(vec!["hello\nworld".to_string()]);
        let result: Vec<_> = lines(source).map(|r| r.unwrap()).collect().await;

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
        let result: Vec<_> = lines(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("hello\n".into()));
        assert_eq!(result[1], Item::Text("world\n".into()));
    }

    #[tokio::test]
    async fn test_lines_bytes_single_line() {
        let source = bytes_source(vec![b"hello\n".to_vec()]);
        let result: Vec<_> = lines(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Bytes(b"hello\n".to_vec().into()));
    }

    #[tokio::test]
    async fn test_lines_bytes_multiple_lines() {
        let source = bytes_source(vec![b"hello\nworld\n".to_vec()]);
        let result: Vec<_> = lines(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Bytes(b"hello\n".to_vec().into()));
        assert_eq!(result[1], Item::Bytes(b"world\n".to_vec().into()));
    }

    #[tokio::test]
    async fn test_lines_bytes_partial_line() {
        let source = bytes_source(vec![b"hello\nworld".to_vec()]);
        let result: Vec<_> = lines(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Bytes(b"hello\n".to_vec().into()));
        assert_eq!(result[1], Item::Bytes(b"world".to_vec().into()));
    }

    #[tokio::test]
    async fn test_lines_bytes_split_across_chunks() {
        let source = bytes_source(vec![b"hel".to_vec(), b"lo\nwor".to_vec(), b"ld\n".to_vec()]);
        let result: Vec<_> = lines(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Bytes(b"hello\n".to_vec().into()));
        assert_eq!(result[1], Item::Bytes(b"world\n".to_vec().into()));
    }

    #[tokio::test]
    async fn test_lines_empty_lines() {
        let source = text_source(vec!["\n\n\n".to_string()]);
        let result: Vec<_> = lines(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Item::Text("\n".into()));
        assert_eq!(result[1], Item::Text("\n".into()));
        assert_eq!(result[2], Item::Text("\n".into()));
    }

    #[tokio::test]
    async fn test_lines_no_newline() {
        let source = text_source(vec!["hello world".to_string()]);
        let result: Vec<_> = lines(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Text("hello world".into()));
    }

    // ===== Transform Tests: chunks() =====

    #[tokio::test]
    async fn test_chunks_text_exact_size() {
        let source = text_source(vec!["12345".to_string()]);
        let result: Vec<_> = chunks(source, 5).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Text("12345".into()));
    }

    #[tokio::test]
    async fn test_chunks_text_multiple_chunks() {
        let source = text_source(vec!["1234567890".to_string()]);
        let result: Vec<_> = chunks(source, 3).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], Item::Text("123".into()));
        assert_eq!(result[1], Item::Text("456".into()));
        assert_eq!(result[2], Item::Text("789".into()));
        assert_eq!(result[3], Item::Text("0".into()));
    }

    #[tokio::test]
    async fn test_chunks_text_partial_chunk() {
        let source = text_source(vec!["12345".to_string()]);
        let result: Vec<_> = chunks(source, 3).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("123".into()));
        assert_eq!(result[1], Item::Text("45".into()));
    }

    #[tokio::test]
    async fn test_chunks_text_across_stream_items() {
        let source = text_source(vec![
            "123".to_string(),
            "456".to_string(),
            "789".to_string(),
        ]);
        let result: Vec<_> = chunks(source, 5).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("12345".into()));
        assert_eq!(result[1], Item::Text("6789".into()));
    }

    #[tokio::test]
    async fn test_chunks_bytes_exact_size() {
        let source = bytes_source(vec![b"12345".to_vec()]);
        let result: Vec<_> = chunks(source, 5).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Bytes(b"12345".to_vec().into()));
    }

    #[tokio::test]
    async fn test_chunks_bytes_multiple_chunks() {
        let source = bytes_source(vec![b"1234567890".to_vec()]);
        let result: Vec<_> = chunks(source, 3).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], Item::Bytes(b"123".to_vec().into()));
        assert_eq!(result[1], Item::Bytes(b"456".to_vec().into()));
        assert_eq!(result[2], Item::Bytes(b"789".to_vec().into()));
        assert_eq!(result[3], Item::Bytes(b"0".to_vec().into()));
    }

    #[tokio::test]
    async fn test_chunks_bytes_partial_chunk() {
        let source = bytes_source(vec![b"12345".to_vec()]);
        let result: Vec<_> = chunks(source, 3).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Bytes(b"123".to_vec().into()));
        assert_eq!(result[1], Item::Bytes(b"45".to_vec().into()));
    }

    #[tokio::test]
    async fn test_chunks_zero_defaults_to_65536() {
        let large_text = "a".repeat(70000);
        let source = text_source(vec![large_text.clone()]);
        let result: Vec<_> = chunks(source, 0).map(|r| r.unwrap()).collect().await;

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
        let result: Vec<_> = chunks(source, 3).map(|r| r.unwrap()).collect().await;

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

        let result: Vec<_> = lines(mixed_source).map(|r| r.unwrap()).collect().await;

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

        let result: Vec<_> = chunks(mixed_source, 3).map(|r| r.unwrap()).collect().await;

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

    // ===== Transform Tests: to_hex() =====

    #[tokio::test]
    async fn test_to_hex_bytes_simple() {
        let source = bytes_source(vec![vec![0xDE, 0xAD, 0xBE, 0xEF]]);
        let result: Vec<_> = to_hex(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Text("deadbeef".into()));
    }

    #[tokio::test]
    async fn test_to_hex_bytes_empty() {
        let source = bytes_source(vec![vec![]]);
        let result: Vec<_> = to_hex(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Text("".into()));
    }

    #[tokio::test]
    async fn test_to_hex_text_ascii() {
        let source = text_source(vec!["hello".to_string()]);
        let result: Vec<_> = to_hex(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Text("68656c6c6f".into()));
    }

    #[tokio::test]
    async fn test_to_hex_text_unicode() {
        let source = text_source(vec!["🦀".to_string()]);
        let result: Vec<_> = to_hex(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Text("f09fa680".into()));
    }

    #[tokio::test]
    async fn test_to_hex_multiple_items() {
        let source = bytes_source(vec![vec![0xAA], vec![0xBB]]);
        let result: Vec<_> = to_hex(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("aa".into()));
        assert_eq!(result[1], Item::Text("bb".into()));
    }

    #[tokio::test]
    async fn test_to_hex_mixed_types() {
        use async_stream::stream;

        let mixed_source = stream! {
            yield Ok(Item::Bytes(vec![0xAA].into()));
            yield Ok(Item::Text("B".into()));
            yield Ok(Item::Bytes(vec![0xCC].into()));
        };

        let result: Vec<_> = to_hex(mixed_source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Item::Text("aa".into()));
        assert_eq!(result[1], Item::Text("42".into())); // 'B' in ASCII
        assert_eq!(result[2], Item::Text("cc".into()));
    }

    // ===== Transform Tests: parse_hex() =====

    #[tokio::test]
    async fn test_parse_hex_text_simple() {
        let source = text_source(vec!["deadbeef".to_string()]);
        let result: Vec<_> = parse_hex(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF].into()));
    }

    #[tokio::test]
    async fn test_parse_hex_text_uppercase() {
        let source = text_source(vec!["DEADBEEF".to_string()]);
        let result: Vec<_> = parse_hex(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF].into()));
    }

    #[tokio::test]
    async fn test_parse_hex_text_mixed_case() {
        let source = text_source(vec!["DeAdBeEf".to_string()]);
        let result: Vec<_> = parse_hex(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF].into()));
    }

    #[tokio::test]
    async fn test_parse_hex_with_whitespace() {
        let source = text_source(vec!["de ad be ef".to_string()]);
        let result: Vec<_> = parse_hex(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF].into()));
    }

    #[tokio::test]
    async fn test_parse_hex_bytes_ascii() {
        let source = bytes_source(vec![b"deadbeef".to_vec()]);
        let result: Vec<_> = parse_hex(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF].into()));
    }

    #[tokio::test]
    async fn test_parse_hex_empty() {
        let source = text_source(vec!["".to_string()]);
        let result: Vec<_> = parse_hex(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Bytes(vec![].into()));
    }

    #[tokio::test]
    async fn test_parse_hex_odd_digits() {
        let source = text_source(vec!["abc".to_string()]);
        let result: Vec<_> = parse_hex(source).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Err("odd number of hex digits"));
    }

    #[tokio::test]
    async fn test_parse_hex_invalid_char() {
        let source = text_source(vec!["abcg".to_string()]);
        let result: Vec<_> = parse_hex(source).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Err("invalid hex digit"));
    }

    #[tokio::test]
    async fn test_parse_hex_all_whitespace() {
        let source = text_source(vec!["   \n\t  ".to_string()]);
        let result: Vec<_> = parse_hex(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Bytes(vec![].into()));
    }

    #[tokio::test]
    async fn test_parse_hex_multiple_items() {
        let source = text_source(vec!["aa".to_string(), "bb".to_string()]);
        let result: Vec<_> = parse_hex(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Bytes(vec![0xAA].into()));
        assert_eq!(result[1], Item::Bytes(vec![0xBB].into()));
    }

    #[tokio::test]
    async fn test_parse_hex_propagates_errors() {
        use async_stream::stream;

        let error_source = stream! {
            yield Ok(Item::Text("aa".into()));
            yield Err("test error");
            yield Ok(Item::Text("bb".into()));
        };

        let result: Vec<_> = parse_hex(error_source).collect().await;

        assert_eq!(result.len(), 3);
        assert!(result[0].is_ok());
        assert_eq!(result[1], Err("test error"));
        assert!(result[2].is_ok());
    }

    // ===== Transform Tests: split_whitespace() =====

    #[tokio::test]
    async fn test_split_whitespace_text_simple() {
        let source = text_source(vec!["hello world".to_string()]);
        let result: Vec<_> = split_whitespace(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("hello".into()));
        assert_eq!(result[1], Item::Text("world".into()));
    }

    #[tokio::test]
    async fn test_split_whitespace_text_multiple_spaces() {
        let source = text_source(vec!["hello    world".to_string()]);
        let result: Vec<_> = split_whitespace(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("hello".into()));
        assert_eq!(result[1], Item::Text("world".into()));
    }

    #[tokio::test]
    async fn test_split_whitespace_text_tabs_newlines() {
        let source = text_source(vec!["hello\t\nworld".to_string()]);
        let result: Vec<_> = split_whitespace(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("hello".into()));
        assert_eq!(result[1], Item::Text("world".into()));
    }

    #[tokio::test]
    async fn test_split_whitespace_text_leading_trailing() {
        let source = text_source(vec!["  hello world  ".to_string()]);
        let result: Vec<_> = split_whitespace(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("hello".into()));
        assert_eq!(result[1], Item::Text("world".into()));
    }

    #[tokio::test]
    async fn test_split_whitespace_text_empty() {
        let source = text_source(vec!["".to_string()]);
        let result: Vec<_> = split_whitespace(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_split_whitespace_text_only_whitespace() {
        let source = text_source(vec!["   \n\t  ".to_string()]);
        let result: Vec<_> = split_whitespace(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_split_whitespace_bytes_simple() {
        let source = bytes_source(vec![b"hello world".to_vec()]);
        let result: Vec<_> = split_whitespace(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Bytes(b"hello".to_vec().into()));
        assert_eq!(result[1], Item::Bytes(b"world".to_vec().into()));
    }

    #[tokio::test]
    async fn test_split_whitespace_across_chunks() {
        let source = text_source(vec![
            "hel".to_string(),
            "lo wor".to_string(),
            "ld".to_string(),
        ]);
        let result: Vec<_> = split_whitespace(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("hello".into()));
        assert_eq!(result[1], Item::Text("world".into()));
    }

    #[tokio::test]
    async fn test_split_whitespace_whitespace_across_chunks() {
        let source = text_source(vec!["hello ".to_string(), " world".to_string()]);
        let result: Vec<_> = split_whitespace(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("hello".into()));
        assert_eq!(result[1], Item::Text("world".into()));
    }

    #[tokio::test]
    async fn test_split_whitespace_partial_word_at_end() {
        let source = text_source(vec!["hello world foo".to_string()]);
        let result: Vec<_> = split_whitespace(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Item::Text("hello".into()));
        assert_eq!(result[1], Item::Text("world".into()));
        assert_eq!(result[2], Item::Text("foo".into()));
    }

    #[tokio::test]
    async fn test_split_whitespace_mixed_types() {
        use async_stream::stream;

        let mixed_source = stream! {
            yield Ok(Item::Text("hello world".into()));
            yield Ok(Item::Bytes(b"foo bar".to_vec().into()));
        };

        let result: Vec<_> = split_whitespace(mixed_source)
            .map(|r| r.unwrap())
            .collect()
            .await;

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], Item::Text("hello".into()));
        assert_eq!(result[1], Item::Text("world".into()));
        assert_eq!(result[2], Item::Bytes(b"foo".to_vec().into()));
        assert_eq!(result[3], Item::Bytes(b"bar".to_vec().into()));
    }

    #[tokio::test]
    async fn test_split_whitespace_propagates_errors() {
        use async_stream::stream;

        let error_source = stream! {
            yield Ok(Item::Text("hello".into()));
            yield Err("test error");
            yield Ok(Item::Text("world".into()));
        };

        let result: Vec<_> = split_whitespace(error_source).collect().await;

        assert_eq!(result.len(), 3);
        assert!(result[0].is_ok());
        assert_eq!(result[1], Err("test error"));
        assert!(result[2].is_ok());
    }

    // ===== Transform Tests: collapse() =====

    #[tokio::test]
    async fn test_collapse_text_simple() {
        let source = text_source(vec!["hello  world".to_string()]);
        let result: Vec<_> = collapse(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Text("hello world".into()));
    }

    #[tokio::test]
    async fn test_collapse_text_multiple_spaces() {
        let source = text_source(vec!["hello    world    foo".to_string()]);
        let result: Vec<_> = collapse(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Text("hello world foo".into()));
    }

    #[tokio::test]
    async fn test_collapse_text_tabs_newlines() {
        let source = text_source(vec!["hello\t\nworld".to_string()]);
        let result: Vec<_> = collapse(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Text("hello world".into()));
    }

    #[tokio::test]
    async fn test_collapse_text_leading_trailing() {
        let source = text_source(vec!["  hello world  ".to_string()]);
        let result: Vec<_> = collapse(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Text("hello world".into()));
    }

    #[tokio::test]
    async fn test_collapse_text_only_whitespace() {
        let source = text_source(vec!["   \n\t  ".to_string()]);
        let result: Vec<_> = collapse(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Text("".into()));
    }

    #[tokio::test]
    async fn test_collapse_bytes_simple() {
        let source = bytes_source(vec![b"hello  world".to_vec()]);
        let result: Vec<_> = collapse(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Bytes(b"hello world".to_vec().into()));
    }

    #[tokio::test]
    async fn test_collapse_multiple_items() {
        let source = text_source(vec!["hello  world".to_string(), "foo   bar".to_string()]);
        let result: Vec<_> = collapse(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("hello world".into()));
        assert_eq!(result[1], Item::Text("foo bar".into()));
    }

    #[tokio::test]
    async fn test_collapse_propagates_errors() {
        use async_stream::stream;

        let error_source = stream! {
            yield Ok(Item::Text("hello  world".into()));
            yield Err("test error");
            yield Ok(Item::Text("foo   bar".into()));
        };

        let result: Vec<_> = collapse(error_source).collect().await;

        assert_eq!(result.len(), 3);
        assert!(result[0].is_ok());
        assert_eq!(result[1], Err("test error"));
        assert!(result[2].is_ok());
    }

    // ===== Transform Tests: filter() =====

    #[tokio::test]
    async fn test_filter_text_mixed() {
        let source = text_source(vec![
            "hello".to_string(),
            "".to_string(),
            "world".to_string(),
        ]);
        let result: Vec<_> = filter(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("hello".into()));
        assert_eq!(result[1], Item::Text("world".into()));
    }

    #[tokio::test]
    async fn test_filter_bytes_mixed() {
        let source = bytes_source(vec![b"hello".to_vec(), vec![], b"world".to_vec()]);
        let result: Vec<_> = filter(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Bytes(b"hello".to_vec().into()));
        assert_eq!(result[1], Item::Bytes(b"world".to_vec().into()));
    }

    #[tokio::test]
    async fn test_filter_all_empty() {
        let source = text_source(vec!["".to_string(), "".to_string()]);
        let result: Vec<_> = filter(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_filter_none_empty() {
        let source = text_source(vec!["hello".to_string(), "world".to_string()]);
        let result: Vec<_> = filter(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("hello".into()));
        assert_eq!(result[1], Item::Text("world".into()));
    }

    #[tokio::test]
    async fn test_filter_propagates_errors() {
        use async_stream::stream;

        let error_source = stream! {
            yield Ok(Item::Text("hello".into()));
            yield Ok(Item::Text("".into()));
            yield Err("test error");
            yield Ok(Item::Text("world".into()));
        };

        let result: Vec<_> = filter(error_source).collect().await;

        assert_eq!(result.len(), 3); // hello, error, world (empty filtered out)
        assert!(result[0].is_ok());
        assert_eq!(result[1], Err("test error"));
        assert!(result[2].is_ok());
    }

    // ===== Transform Tests: to_binary() =====

    #[tokio::test]
    async fn test_to_binary_bytes_simple() {
        let source = bytes_source(vec![vec![0xFF, 0x00, 0xAA]]);
        let result: Vec<_> = to_binary(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Text("111111110000000010101010".into()));
    }

    #[tokio::test]
    async fn test_to_binary_bytes_empty() {
        let source = bytes_source(vec![vec![]]);
        let result: Vec<_> = to_binary(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Text("".into()));
    }

    #[tokio::test]
    async fn test_to_binary_text_ascii() {
        let source = text_source(vec!["AB".to_string()]);
        let result: Vec<_> = to_binary(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        // 'A' = 0x41 = 01000001, 'B' = 0x42 = 01000010
        assert_eq!(result[0], Item::Text("0100000101000010".into()));
    }

    #[tokio::test]
    async fn test_to_binary_multiple_items() {
        let source = bytes_source(vec![vec![0x01], vec![0x02]]);
        let result: Vec<_> = to_binary(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Text("00000001".into()));
        assert_eq!(result[1], Item::Text("00000010".into()));
    }

    #[tokio::test]
    async fn test_to_binary_propagates_errors() {
        use async_stream::stream;

        let error_source = stream! {
            yield Ok(Item::Bytes(vec![0xFF].into()));
            yield Err("test error");
        };

        let result: Vec<_> = to_binary(error_source).collect().await;

        assert_eq!(result.len(), 2);
        assert!(result[0].is_ok());
        assert_eq!(result[1], Err("test error"));
    }

    // ===== Transform Tests: parse_binary() =====

    #[tokio::test]
    async fn test_parse_binary_text_simple() {
        let source = text_source(vec!["1111111100000000".to_string()]);
        let result: Vec<_> = parse_binary(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Bytes(vec![0xFF, 0x00].into()));
    }

    #[tokio::test]
    async fn test_parse_binary_text_with_whitespace() {
        let source = text_source(vec!["11111111 00000000".to_string()]);
        let result: Vec<_> = parse_binary(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Bytes(vec![0xFF, 0x00].into()));
    }

    #[tokio::test]
    async fn test_parse_binary_bytes_ascii() {
        let source = bytes_source(vec![b"0100000101000010".to_vec()]);
        let result: Vec<_> = parse_binary(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        // Should be 'A' (0x41) and 'B' (0x42)
        assert_eq!(result[0], Item::Bytes(vec![0x41, 0x42].into()));
    }

    #[tokio::test]
    async fn test_parse_binary_empty() {
        let source = text_source(vec!["".to_string()]);
        let result: Vec<_> = parse_binary(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Bytes(vec![].into()));
    }

    #[tokio::test]
    async fn test_parse_binary_all_whitespace() {
        let source = text_source(vec!["  \n\t  ".to_string()]);
        let result: Vec<_> = parse_binary(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Bytes(vec![].into()));
    }

    #[tokio::test]
    async fn test_parse_binary_invalid_bit_count() {
        let source = text_source(vec!["1111111".to_string()]); // 7 bits
        let result: Vec<_> = parse_binary(source).collect().await;

        assert_eq!(result.len(), 1);
        assert!(result[0].is_err());
        assert_eq!(result[0], Err("binary string bit count not multiple of 8"));
    }

    #[tokio::test]
    async fn test_parse_binary_invalid_digit() {
        let source = text_source(vec!["11111112".to_string()]);
        let result: Vec<_> = parse_binary(source).collect().await;

        assert_eq!(result.len(), 1);
        assert!(result[0].is_err());
        assert_eq!(result[0], Err("invalid binary digit"));
    }

    #[tokio::test]
    async fn test_parse_binary_multiple_items() {
        let source = text_source(vec!["11111111".to_string(), "00000000".to_string()]);
        let result: Vec<_> = parse_binary(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Bytes(vec![0xFF].into()));
        assert_eq!(result[1], Item::Bytes(vec![0x00].into()));
    }

    #[tokio::test]
    async fn test_parse_binary_propagates_errors() {
        use async_stream::stream;

        let error_source = stream! {
            yield Ok(Item::Text("11111111".into()));
            yield Err("test error");
            yield Ok(Item::Text("00000000".into()));
        };

        let result: Vec<_> = parse_binary(error_source).collect().await;

        assert_eq!(result.len(), 3);
        assert!(result[0].is_ok());
        assert_eq!(result[1], Err("test error"));
        assert!(result[2].is_ok());
    }
    // ===== Transform Tests: split_shell() =====

    #[tokio::test]
    async fn test_split_shell_simple() {
        let source = text_source(vec!["hello world".to_string()]);
        let result: Vec<_> = split_shell(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Bytes(b"hello".to_vec().into()));
        assert_eq!(result[1], Item::Bytes(b"world".to_vec().into()));
    }

    #[tokio::test]
    async fn test_split_shell_quoted() {
        let source = text_source(vec!["hello 'world foo'".to_string()]);
        let result: Vec<_> = split_shell(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Bytes(b"hello".to_vec().into()));
        assert_eq!(result[1], Item::Bytes(b"world foo".to_vec().into()));
    }

    #[tokio::test]
    async fn test_split_shell_double_quoted() {
        let source = text_source(vec!["hello \"world foo\"".to_string()]);
        let result: Vec<_> = split_shell(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Bytes(b"hello".to_vec().into()));
        assert_eq!(result[1], Item::Bytes(b"world foo".to_vec().into()));
    }

    #[tokio::test]
    async fn test_split_shell_escaped() {
        let source = text_source(vec!["hello\\ world".to_string()]);
        let result: Vec<_> = split_shell(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::Bytes(b"hello world".to_vec().into()));
    }

    #[tokio::test]
    async fn test_split_shell_bytes() {
        let source = bytes_source(vec![b"hello world".to_vec()]);
        let result: Vec<_> = split_shell(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Item::Bytes(b"hello".to_vec().into()));
        assert_eq!(result[1], Item::Bytes(b"world".to_vec().into()));
    }

    #[tokio::test]
    async fn test_split_shell_multiple_items() {
        let source = text_source(vec!["hello world".to_string(), "foo bar".to_string()]);
        let result: Vec<_> = split_shell(source).map(|r| r.unwrap()).collect().await;

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], Item::Bytes(b"hello".to_vec().into()));
        assert_eq!(result[1], Item::Bytes(b"world".to_vec().into()));
        assert_eq!(result[2], Item::Bytes(b"foo".to_vec().into()));
        assert_eq!(result[3], Item::Bytes(b"bar".to_vec().into()));
    }

    #[tokio::test]
    async fn test_split_shell_propagates_errors() {
        use async_stream::stream;

        let error_source = stream! {
            yield Ok(Item::Text("hello world".into()));
            yield Err("test error");
            yield Ok(Item::Text("foo bar".into()));
        };

        let result: Vec<_> = split_shell(error_source).collect().await;

        assert_eq!(result.len(), 5); // 2 + error + 2
        assert!(result[0].is_ok());
        assert!(result[1].is_ok());
        assert_eq!(result[2], Err("test error"));
        assert!(result[3].is_ok());
        assert!(result[4].is_ok());
    }
}
