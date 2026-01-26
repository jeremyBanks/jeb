
#[deprecated]
macro_rules! load {
    {$($tt:tt)*} => {
        {todo!()}
    }
}

#[instrument]
async fn async_main() -> Result<(), panic> {

    let ryl_fic_ids = [
        21220, 22518, 25137, 35858, 36950, 45534, 48948, 49033, 60396,
    ];

    tokio::fs::remove_file("data/spines/index.json").await.ok();
    // let index = load!("data/spines/index", async move || {
    //     futures::future::join_all(ryl_fic_ids.map(royalroad::fic))
    //         .await
    //         .into_iter()
    //         .collect::<Result<Vec<_>, _>>()?
    //         .into_iter()
    //         .map(
    //             |royalroad::Fic {
    //                  ref id10,
    //                  ref title,
    //                  ..
    //              }| json! {{ id10: id10, title: title }},
    //         )
    //         .collect::<Vec<_>>()
    // })?;

    // let speech = wavs_to_opus(vec![
    //     speak_as(
    //         "Chrysalis, by Rhino Z... Chapter 85: The Egg and the Serpent",
    //         "Microsoft Zira",
    //     )
    //     .await?,
    //     speak_as(
    //         "hello, world! said the egg. and that was the end.",
    //         "Microsoft Richard",
    //     )
    //     .await?,
    // ])?;

    Ok::<(), panic>(())
}


mod web {
    use super::*;

    static THROTTLE: Lazy<Throttle> = Lazy::new(|| throttle("web", 4 * 1024));

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Page {
        pub url:          String,
        pub url_final:    String,
        pub content_type: Option<String>,
        pub body:         String,
    }

    #[instrument(level = "trace", skip_all)]
    pub async fn get(url: impl AsRef<str>) -> Result<Page, panic> {
        let url = url.as_ref().to_string();
        let digest = digest(url.as_bytes());
        load!("target/web/{digest}", async move || {
            THROTTLE.tick().await;

            info!("Fetching {url}");
            let request = reqwest::get(url.to_string());
            let response = request.await?.error_for_status()?;
            let content_type =
                if let Some(header) = response.headers().get(http::header::CONTENT_TYPE) {
                    Some(header.to_str()?.to_string())
                } else {
                    None
                };
            let url_final = response.url().to_string();
            let body = response.bytes().await?.to_vec();
            let body = String::from_utf8_lossy(&body).to_string();
            Page {
                body,
                content_type,
                url: url.into(),
                url_final: url_final.into(),
            }
        })
    }
}

mod royalroad {
    use super::*;

    static THROTTLE: Lazy<Throttle> = Lazy::new(|| throttle("RoyalRoad", 128));

    #[instrument(level = "trace")]
    pub async fn spine(id: u64) -> Result<Spine, panic> {
        let id10 = fic_id10(id);
        let spine = load!("target/spines/{id10}", async move || {
            THROTTLE.tick().await;

            let url = f!["https://www.royalroad.com/fiction/{id}"];

            let page = web::get(url).await?;

            let html = page.body;

            let document = Html::parse_document(html.as_ref());

            let mut chapters = BTreeSet::new();

            let title = document
                .select(&Selector::parse("title").unwrap())
                .next()
                .unwrap()
                .text()
                .next()
                .unwrap()
                .split("|")
                .next()
                .unwrap()
                .trim()
                .to_owned()
                .into();

            static FORMAT_1: Lazy<
                Result<Vec<format_description::FormatItem>, InvalidFormatDescription>,
            > = Lazy::new(|| {
                format_description::parse(
                    "[weekday], [day] [month repr:long] [year] [hour]:[minute]",
                )
            });
            static FORMAT_2: Lazy<
                Result<Vec<format_description::FormatItem>, InvalidFormatDescription>,
            > = Lazy::new(|| {
                format_description::parse(
                    "[weekday], [month repr:long] [day padding:none], [year] [hour repr:12 \
                     padding:none]:[minute] [period]",
                )
            });

            for chapter in document.select(&Selector::parse("table#chapters tbody tr")?) {
                let html = chapter.html();

                let chapter_link = chapter.select(&Selector::parse("a")?).next().unwrap();
                let chapter_time = chapter.select(&Selector::parse("time")?).next().unwrap();

                let timestamp: i64 = chapter_time
                    .value()
                    .attr("unixtime")
                    .map(|s| s.parse().unwrap())
                    .unwrap_or_else(|| {
                        let s = chapter_time
                            .value()
                            .attr("title")
                            .expect("no title found in chapter link, either!");
                        PrimitiveDateTime::parse(s, FORMAT_1.as_ref().unwrap())
                            .or(PrimitiveDateTime::parse(s, FORMAT_2.as_ref().unwrap()))
                            .expect(&f!["{s:?}"])
                            .assume_utc()
                            .unix_timestamp()
                    });

                let title = chapter_link
                    .text()
                    .next()
                    .unwrap()
                    .trim()
                    .to_string()
                    .into();
                let href = chapter_link.value().attr("href").unwrap();

                let mut id_slug = href
                    .split("://")
                    .last()
                    .unwrap()
                    .split("/chapter/")
                    .last()
                    .unwrap()
                    .split("/");
                let id = id_slug.next().unwrap().parse()?;

                chapters.insert(SpineChapter {
                    id,
                    id10: chapter_id10(id),
                    timestamp,
                    title,
                });
            }

            Spine {
                id,
                id10: fic_id10(id),
                title,
                chapters,
            }
        });

        Ok(spine)
    }

    #[instrument(level = "trace")]
    pub async fn fic(id: u64) -> Result<Fic, panic> {
        let id10 = fic_id10(id);

        let fic = load!("target/fics/{id10}", async move || {
            let spine = spine(id).await?;

            let mut chapters = BTreeSet::new();

            for chapter in &spine.chapters {
                let chapter = fic_chapter(&spine, &chapter).await?;
                chapters.insert(chapter);
            }

            let fic = Fic {
                id,
                id10: fic_id10(id),
                title: spine.title,
                chapters,
            };

            info!("Loaded fic #{id} {title:?}", title = &fic.title);
            info!(chapter_count = fic.chapters.len());

            fic
        });

        // let ff = ;
        // let _rich: Result<RichSpine, _> = load!("data/spines/{id10}", async move || {
        //     let mut chapters = BTreeSet::new();

        //     for chapter in ff.chapters {
        //         let starts_with = ammonia::clean(
        //             &(chapter
        //                 .html
        //                 .to_string()
        //                 .split_ascii_whitespace()
        //                 .collect::<Vec<_>>()
        //                 .join(" ")
        //                 .chars()
        //                 .take(255)
        //                 .collect::<String>()
        //                 .rsplit_once(" ")
        //                 .unwrap()
        //                 .0
        //                 .to_string()
        //                 + "…"),
        //         );

        //         let chapter = RichSpineChapter {
        //             id10: chapter.id10.clone(),
        //             timestamp: chapter.timestamp,
        //             title: chapter.title.clone(),
        //             length: chapter.html.len() as _,
        //             starts_with,
        //         };
        //         chapters.insert(chapter);
        //     }

        //     RichSpine {
        //         id10,
        //         author: "TODO".to_string(),
        //         title: ff.title,
        //         length: chapters.iter().map(|c| c.length).sum(),
        //         chapters,
        //     }
        // });

        Ok(fic)
    }

    pub async fn fic_chapter(spine: &Spine, chapter: &SpineChapter) -> Result<FicChapter, panic> {
        let spine = spine.clone();
        let chapter = chapter.clone();
        let fic_id = spine.id;
        let chapter_id = chapter.id;

        let fic10 = fic_id10(fic_id);
        let id10 = chapter_id10(chapter_id);

        load!("target/chapters/{fic10}{id10}", async move || {
            THROTTLE.tick().await;

            let url = f!["https://www.royalroad.com/fiction/{fic_id}/_/chapter/{chapter_id}/_"];

            let html = web::get(url).await?.body;

            let document = Html::parse_document(html.as_ref());

            let html_original = document
                .select(&Selector::parse("div.chapter-inner")?)
                .next()
                .expect("missing expected div.chapter-inner in document")
                .html();

            let html = ammonia::Builder::new()
                .rm_tags(HashSet::<&str>::from_iter(["img", "span"]))
                .url_schemes(HashSet::<&str>::from_iter([
                    "http", "https", "mailto", "magnet",
                ]))
                .url_relative(ammonia::UrlRelative::Deny)
                .clean(&html_original)
                .to_string()
                .into();

            FicChapter {
                id: chapter.id,
                id10: chapter_id10(chapter.id),
                title: chapter.title.clone(),
                timestamp: chapter.timestamp,
                html,
            }
        })
    }

    fn fic_id10(id: u64) -> String {
        format!("RYL{id:07}")
    }

    fn chapter_id10(id: u64) -> String {
        format!("C{id:09}")
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Fic {
        pub id:       u64,
        pub id10:     String,
        pub title:    String,
        pub chapters: BTreeSet<FicChapter>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct FicChapter {
        pub id:        u64,
        pub id10:      String,
        pub timestamp: i64,
        pub title:     String,
        pub html:      String,
    }
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Spine {
        pub id:       u64,
        pub id10:     String,
        pub title:    String,
        pub chapters: BTreeSet<SpineChapter>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SpineChapter {
        pub id:        u64,
        pub id10:      String,
        pub timestamp: i64,
        pub title:     String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RichSpine {
        pub id10:     String,
        pub title:    String,
        pub author:   String,
        pub length:   u64,
        pub chapters: BTreeSet<RichSpineChapter>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RichSpineChapter {
        pub id10:        String,
        pub timestamp:   i64,
        pub title:       String,
        pub length:      u64,
        pub starts_with: String,
    }
}
