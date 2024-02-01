pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Bar {
    origin: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Message {
    role: String,
    content: String,
    images: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
enum Response {
    Data {
        model: String,
        created_at: String,
        message: Message,
        done: bool,
    },

    Done {
        model: String,
        created_at: String,
        eval_count: usize,
        done: bool,
    },
}

#[cfg(test)]
mod tests {
    use std::process::Output;

    use super::*;
    use futures_util::{Stream, StreamExt};

    // Implementation of mock
    // ollama client for tests

    #[tokio::test]
    async fn test_ollama_chat_api() {
        let line = "The quick brown fox jumps over a lazy dog";
        let mut s = line.split_inclusive(' ');

        let data = core::iter::from_fn(|| {
            Some(Ok::<_, std::io::Error>(
                (serde_json::json!({
                  "model": "llama2",
                  "created_at": "2023-08-04T08:52:19.385406455-07:00",
                  "message": {
                    "role": "assistant",
                    "content": s.next()?,
                  },
                  "done": false
                })
                .to_string()
                    + "\n")
                    .into_bytes(),
            ))
        })
        .chain(core::iter::once(Ok((serde_json::json!({
          "model": "llama2",
          "created_at": "2023-08-04T19:22:45.499127Z",
          "eval_count": 282,
          "done": true,
        })
        .to_string()
            + "\n")
            .into_bytes())));

        let stream = futures_util::stream::iter(data);

        let mut stream = json_stream::JsonStream::<Response, _>::new(stream);

        while let Some(foo) = stream.next().await {
            println!("{:?}", foo);
        }
    }

    // #[tokio::test]
    async fn test_stream() {
        let stream = reqwest::get("http://httpbin.org/ip")
            .await
            .unwrap()
            .bytes_stream();

        let mut stream = json_stream::JsonStream::<Bar, _>::new(stream);

        while let Some(item) = stream.next().await {
            println!("Chunk: {:?}", item.unwrap());
        }
    }

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
