use super::*;

pub trait Load:
    DeserializeOwned + Serialize + Clone + Debug + Send + Sync + PartialEq + 'static
{
}

impl<T> Load for T where
    T: DeserializeOwned + Serialize + Clone + Debug + Send + Sync + PartialEq + 'static
{
}

#[instrument(level = "trace", skip_all)]
pub async fn load<Output: Load>(
    path: Option<&Path>,
    fetch: impl FnOnce() -> tokio::task::JoinHandle<Result<Output, eyre::Report>>,
) -> Result<Output, eyre::Report> {
    if let Some(path) = path {
        trace!("Loading {path:?}");
        match fs::read(path).await {
            Ok(bytes) => match serde_json::from_slice(&bytes) {
                Ok(output) => {
                    debug!("Loaded {path:?}");
                    return Ok(output);
                },
                Err(err) => {
                    warn!("Found existing file at {path:?} but parsing failed: {err}");
                },
            },
            Err(err) => {
                info!("Failed to read existing file at {path:?}: {err}");
            },
        }
    }

    info!("Fetching {path:?}...");
    let fetched = fetch().await.wrap()??;

    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut bytes = serde_json::to_vec_pretty(&fetched).wrap()?;
        bytes.push(b'\n');

        info!("Saving fetched value to {path:?}");
        fs::write(path, &bytes).await.wrap()?;
    }

    Ok(fetched)
}

#[macro_export(crate)]
macro_rules! load {
($path:expr, async $($move:ident)? || $( -> $output:path)? { $($body:tt)* }) => {
    {
        let path = format!($path) + ".json";
        let path = Path::new(&path);
        let output$(: $output)? = load(
            Some(path),
            $($move)? || tokio::spawn(async $($move)? { Ok({ $($body)* }) }),
        ).await?;
        Ok::<_, eyre::Report>(output)
    }
};

(async $($move:ident)? || $( -> $output:path)? { $($body:tt)* }) => {
    {
        let output$(: $output)? = load(
            None,
            $($move)? || tokio::spawn(async $($move)? { Ok({ $($body)* }) }),
        ).await?;
        Ok::<_, eyre::Report>(output)
    }
};
}
