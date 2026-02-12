use minidom::Element;
use serde_derive::{Deserialize, Serialize};
use std::{collections::BTreeSet, convert::TryFrom, env, fs, string::ToString, time::Duration};
use tracing_unwrap::OptionExt;
use tracing_unwrap::ResultExt;

use crate::{domutils::*, durationutils::*, stringutils::*};

fn old_main() {
    let saves = env::args()
        .skip(1)
        .map(|name| fs::read_to_string(name).expect("file should exist"))
        .collect::<Vec<_>>();

    if saves.is_empty() {
        eprintln!("Error: no arguments provided. One or more Celeste save file paths expected.");
        return;
    }

    const HEADER_FG: AnsiColor = Black;
    const HEADER_BG: AnsiColor = White;
    const DIVIDER: AnsiColor = DarkGray;

    const IRRELEVANT: AnsiColor = DarkGray;
    const SUBPAR: AnsiColor = DarkRed;
    const NORMAL: AnsiColor = White;
    const GOOD: AnsiColor = Magenta;
    const BEST: AnsiColor = Yellow;

    fn print_divider(content: impl ToString) {
        let mut s = format!("  {:<69}", content.to_string());

        s = s.color(HEADER_FG).background(HEADER_BG);

        println!("{}", s);
    }

    fn print_side(side: impl ToString, color: AnsiColor) {
        print!("{} ", " ".background(DIVIDER));
        print!("{}", side.to_string().color(color));
        print!(" {}", " ".background(DIVIDER));
    }

    fn print_cell(left: impl ToString, right: impl ToString, color: AnsiColor, max_len: usize) {
        let left = left.to_string();
        let right = right.to_string();
        let content_len = left.len() + right.len();

        let mut s = String::new();
        if content_len <= max_len || true {
            let padding = if content_len < max_len {
                max_len - left.len() - right.len()
            } else {
                0
            };
            s.push_str(&left);
            for _ in 0..padding {
                s.push(' ');
            }
            s.push_str(&right);
        } else {
            let mut left = &left[..];
            let mut right = &right[..];
            let mut content_len = content_len;
            while content_len > max_len {
                if left.is_empty() || (!right.is_empty() && content_len % 2 == 0) {
                    right = &right[..right.len() - 1];
                } else {
                    left = &left[..left.len() - 1];
                }
                content_len -= 1;
            }
            s.push_str(left);
            s.push_str(right);
        }

        let force_color = env::var("CELESTE_SAVE_COLOR")
            .map(|s| s == "ON")
            .unwrap_or(false);
        if force_color || atty::is(atty::Stream::Stdout) {
            print!(" {}", s.color(color));
            print!(" {}", " ".background(DIVIDER));
        } else {
            print!(" {}  ", s);
        }
    }

    fn print_time_or_reds(left: impl ToString, right: impl ToString, color: AnsiColor) {
        print_cell(left, right, color, 19);
    }

    fn print_dashes_or_cassette(left: impl ToString, right: impl ToString, color: AnsiColor) {
        print_cell(left, right, color, 19);
    }

    fn print_deaths_or_heart(left: impl ToString, right: impl ToString, color: AnsiColor) {
        print_cell(left, right, color, 19);
        let force_color = env::var("CELESTE_SAVE_COLOR")
            .map(|s| s == "ON")
            .unwrap_or(false);
        if force_color || atty::is(atty::Stream::Stdout) {
            println!("\x1B[0m");
        } else {
            println!();
        }
    }

    for save in saves {
        let root = save.parse::<Element>().unwrap_or_log();
        let stats = Stats::from_save(&root);

        let berry_color = match stats.total_berries {
            0 => SUBPAR,
            1..=174 => NORMAL,
            175..=199 => GOOD,
            200 => BEST,
            _ => panic!("more than 200 berries"),
        };

        let force_color = env::var("CELESTE_SAVE_COLOR")
            .map(|s| s == "ON")
            .unwrap_or(false);
        if force_color || atty::is(atty::Stream::Stdout) {
            println!(
                " {} {}",
                stats.name.underline().color(White),
                format!("{}🍓", stats.total_berries).color(berry_color)
            );
        } else {
            println!(" {} {}🍓", stats.name, stats.total_berries);
        }

        for world_stats in stats.worlds {
            if !(world_stats.a_side.common.completed
                || world_stats.b_side.common.completed
                || world_stats.c_side.common.completed)
            {
                continue;
            }

            print_divider(world_stats.world);

            if world_stats.world == Prologue {
                let duration = world_stats.a_side.common.single_run.unwrap_or_log();
                print_side("p", IRRELEVANT);
                print_time_or_reds("any%:", duration.formatted(), NORMAL);
                print_dashes_or_cassette("can't dash", "", IRRELEVANT);
                let min_deaths = world_stats.a_side.common.fewest_deaths.unwrap_or_log();
                print_deaths_or_heart(
                    "min deaths:",
                    format!("{:>4}", min_deaths),
                    if min_deaths > 0 { NORMAL } else { BEST },
                );
                continue;
            }
            if world_stats.world == Epilogue {
                print_side("e", IRRELEVANT);
                print_time_or_reds("not timed", "", IRRELEVANT);
                let min_dashes = world_stats.a_side.common.fewest_dashes.unwrap_or_log();
                print_dashes_or_cassette(
                    "min dashes:",
                    format!("{:>4}", min_dashes),
                    if min_dashes > 0 { NORMAL } else { BEST },
                );
                print_deaths_or_heart("can't die", "", IRRELEVANT);
                continue;
            }

            if world_stats.a_side.common.completed {
                print_side("A", NORMAL);

                if let Some(duration) = world_stats.a_side.common.single_run {
                    print_time_or_reds("any%:", duration.formatted(), NORMAL);

                    if !world_stats.has_winged_golden() {
                        let min_dashes = world_stats.a_side.common.fewest_dashes.unwrap_or_log();
                        print_dashes_or_cassette(
                            "min dashes:",
                            format!("{:>4}", min_dashes),
                            if min_dashes > 0 { NORMAL } else { BEST },
                        );
                    } else {
                        print_dashes_or_cassette("has winged berry", "", BEST);
                    }

                    if !world_stats.has_golden_a() {
                        let min_deaths = world_stats.a_side.common.fewest_deaths.unwrap_or_log();
                        print_deaths_or_heart(
                            "min deaths:",
                            format!("{:>4}", min_deaths),
                            if min_deaths > 0 { NORMAL } else { BEST },
                        );
                    } else {
                        print_deaths_or_heart("has golden berry", "", BEST);
                    }
                } else {
                    print_time_or_reds("segmented", "", SUBPAR);
                    print_dashes_or_cassette("segmented", "", SUBPAR);
                    print_deaths_or_heart("segmented", "", SUBPAR);
                }

                if world_stats.world.has_unlockables() {
                    print_side("A", NORMAL);

                    if let Some(duration) = world_stats.a_side.full_clear {
                        print_time_or_reds("full:", duration.formatted(), BEST);
                        if world_stats.world == Core {
                            print_dashes_or_cassette("can't skip cassette", "", IRRELEVANT);
                            print_deaths_or_heart("can't skip heart", "", IRRELEVANT);
                        } else {
                            print_dashes_or_cassette("has cassette", "", BEST);
                            print_deaths_or_heart("has crystal heart", "", BEST);
                        }
                    } else {
                        if world_stats.world.red_berries() > 0 {
                            print_time_or_reds(
                                if world_stats.world.red_berries() < 99 {
                                    format!(
                                        "{:>2} / {:<2}",
                                        world_stats.red_berries(),
                                        world_stats.world.red_berries()
                                    )
                                } else {
                                    format!(
                                        "{:>3}/{:<3}",
                                        world_stats.red_berries(),
                                        world_stats.world.red_berries()
                                    )
                                },
                                "red berries",
                                if world_stats.red_berries() > 0 {
                                    if world_stats.red_berries() >= world_stats.world.red_berries()
                                    {
                                        GOOD
                                    } else {
                                        NORMAL
                                    }
                                } else {
                                    SUBPAR
                                },
                            );
                        } else {
                            print_time_or_reds("no red berries here", "", IRRELEVANT);
                        }

                        if world_stats.world == Core {
                            print_dashes_or_cassette("can't skip cassette", "", IRRELEVANT);
                        } else if world_stats.a_side.cassette {
                            print_dashes_or_cassette("has cassette", "", GOOD);
                        } else {
                            print_dashes_or_cassette("no cassette", "", NORMAL);
                        }

                        if world_stats.world == Core {
                            print_deaths_or_heart("can't skip heart", "", IRRELEVANT);
                        } else if world_stats.a_side.heart {
                            print_deaths_or_heart("has crystal heart", "", GOOD);
                        } else if world_stats.world == TheSummit {
                            print_deaths_or_heart(
                                format!("{} / 6 heart gems", stats.gems),
                                "",
                                if stats.gems > 0 { NORMAL } else { SUBPAR },
                            );
                        } else {
                            print_deaths_or_heart("no crystal heart", "", NORMAL);
                        }
                    }
                }
            }

            if world_stats.b_side.common.completed {
                print_side("B", GOOD);

                if let Some(duration) = world_stats.b_side.common.single_run {
                    print_time_or_reds("any%:", duration.formatted(), NORMAL);

                    let min_dashes = world_stats.b_side.common.fewest_dashes.unwrap_or_log();
                    print_dashes_or_cassette(
                        "min dashes:",
                        format!("{:>4}", min_dashes),
                        if min_dashes > 0 { NORMAL } else { BEST },
                    );

                    if !world_stats.has_golden_b() {
                        let min_deaths = world_stats.b_side.common.fewest_deaths.unwrap_or_log();
                        print_deaths_or_heart(
                            "min deaths:",
                            format!("{:>4}", min_deaths),
                            if min_deaths > 0 { NORMAL } else { BEST },
                        );
                    } else {
                        print_deaths_or_heart("has golden berry", "", BEST);
                    }
                } else {
                    print_time_or_reds("segmented", "", SUBPAR);
                    print_dashes_or_cassette("segmented", "", SUBPAR);
                    print_deaths_or_heart("segmented", "", SUBPAR);
                }
            }

            if world_stats.c_side.common.completed {
                print_side("C", BEST);

                if let Some(duration) = world_stats.c_side.common.single_run {
                    print_time_or_reds("any%:", duration.formatted(), NORMAL);

                    let min_dashes = world_stats.c_side.common.fewest_dashes.unwrap_or_log();
                    print_dashes_or_cassette(
                        "min dashes:",
                        format!("{:>4}", min_dashes),
                        if min_dashes > 0 { NORMAL } else { BEST },
                    );

                    if !world_stats.has_golden_c() {
                        let min_deaths = world_stats.c_side.common.fewest_deaths.unwrap_or_log();
                        print_deaths_or_heart(
                            "min deaths:",
                            format!("{:>4}", min_deaths),
                            if min_deaths > 0 { NORMAL } else { BEST },
                        );
                    } else {
                        print_deaths_or_heart("has golden berry", "", BEST);
                    }
                } else {
                    print_time_or_reds("segmented", "", SUBPAR);
                    print_dashes_or_cassette("segmented", "", SUBPAR);
                    print_deaths_or_heart("segmented", "", SUBPAR);
                }
            }
        }

        print_divider("");
        println!();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub version: String,
    pub cheat_mode: bool,
    pub assist_mode: bool,
    pub variant_mode: bool,
    pub name: String,
    pub worlds: Vec<WorldStats>,
    pub total_berries: u32,
    pub gems: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldStats {
    pub world: World,
    pub a_side: ASideStats,
    pub b_side: BSideStats,
    pub c_side: CSideStats,
}

impl WorldStats {
    pub fn red_berries(&self) -> u32 {
        let actual = self.a_side.common.berry_count();
        let max = self.world.red_berries();
        if actual <= max {
            actual
        } else if actual == max + 1 {
            max
        } else if self.world == ForsakenCity && actual == max + 2 {
            max
        } else {
            panic!("impossibly large number of berries")
        }
    }

    pub fn has_golden_a(&self) -> bool {
        self.a_side.common.berry_count() > self.world.red_berries()
    }

    pub fn has_golden_b(&self) -> bool {
        self.b_side.common.berry_count() > 0
    }

    pub fn has_golden_c(&self) -> bool {
        self.c_side.common.berry_count() > 0
    }

    pub fn has_winged_golden(&self) -> bool {
        self.world == ForsakenCity
            && self.a_side.common.berry_count() > self.world.red_berries() + 1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideStatsCommon {
    pub completed: bool,
    pub single_run: Option<Duration>,
    pub fewest_dashes: Option<u32>,
    pub fewest_deaths: Option<u32>,
    pub berries: BTreeSet<String>,
}

impl SideStatsCommon {
    pub fn berry_count(&self) -> u32 {
        self.berries.len() as u32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASideStats {
    pub cassette: bool,
    pub heart: bool,
    pub full_clear: Option<Duration>,
    pub common: SideStatsCommon,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BSideStats {
    pub common: SideStatsCommon,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSideStats {
    pub common: SideStatsCommon,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum World {
    Prologue,
    ForsakenCity,
    OldSite,
    CelestialResort,
    GoldenRidge,
    MirrorTemple,
    Reflection,
    TheSummit,
    Epilogue,
    Core,
    Farewell,
    SumOfBests,
}

pub use self::World::*;

impl World {
    pub fn name(self) -> &'static str {
        match self {
            Prologue => "    Prologue",
            ForsakenCity => "1.  Forsaken City",
            OldSite => "2.  Old Site",
            CelestialResort => "3.  Celestial Resort",
            GoldenRidge => "4.  Golden Ridge",
            MirrorTemple => "5.  Mirror Temple",
            Reflection => "6.  Reflection",
            TheSummit => "7.  The Summit",
            Epilogue => "    Epilogue",
            Core => "8.  Core",
            Farewell => "9.  Farewell",
            SumOfBests => "    Sum of Bests",
        }
    }

    pub fn has_unlockables(self) -> bool {
        match self {
            Prologue | Epilogue => false,
            _ => true,
        }
    }

    pub fn red_berries(self) -> u32 {
        match self {
            Prologue | Reflection | Epilogue => 0,
            ForsakenCity => 20,
            OldSite => 18,
            CelestialResort => 25,
            GoldenRidge => 29,
            MirrorTemple => 31,
            TheSummit => 47,
            Core => 5,
            Farewell => 0,
            SumOfBests => 20 + 18 + 25 + 29 + 31 + 47 + 5,
        }
    }
}

impl std::fmt::Display for World {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl From<u32> for World {
    fn from(id: u32) -> Self {
        match id {
            0 => Prologue,
            1 => ForsakenCity,
            2 => OldSite,
            3 => CelestialResort,
            4 => GoldenRidge,
            5 => MirrorTemple,
            6 => Reflection,
            7 => TheSummit,
            8 => Epilogue,
            9 => Core,
            10 => Farewell,
            100 => SumOfBests,
            _ => panic!("unknown world ID"),
        }
    }
}

impl From<World> for u32 {
    fn from(val: World) -> Self {
        match val {
            Prologue => 0,
            ForsakenCity => 1,
            OldSite => 2,
            CelestialResort => 3,
            GoldenRidge => 4,
            MirrorTemple => 5,
            Reflection => 6,
            TheSummit => 7,
            Epilogue => 8,
            Core => 9,
            Farewell => 10,
            SumOfBests => 100,
        }
    }
}

impl Stats {
    pub fn from_save(save_data: &minidom::Element) -> Self {
        assert!(save_data.name() == "SaveData");

        let version = save_data.expect_child("Version").text();

        let name = save_data.expect_child("Name").text();

        let gem_el = save_data.children().find(|el| el.name() == "SummitGems");

        let gems = match gem_el {
            Some(el) => u8::try_from(el.children().filter(|el| el.text() == "true").count())
                .expect("way too many gems"),
            None => 0,
        };

        let cheat_mode = save_data.expect_parse_child("CheatMode");
        let assist_mode = save_data.expect_parse_child("AssistMode");
        let variant_mode = save_data.expect_parse_child("VariantMode");

        let total_berries = save_data.expect_parse_child("TotalStrawberries");

        let mut worlds: Vec<_> = save_data
            .expect_child("Areas")
            .children()
            .map(WorldStats::from_save)
            .filter(|stats| stats.world != Epilogue)
            .collect();

        worlds.push(WorldStats {
            world: SumOfBests,
            a_side: ASideStats {
                cassette: worlds
                    .iter()
                    .filter(|world_stats| world_stats.world.has_unlockables())
                    .all(|world_stats| world_stats.a_side.cassette),
                full_clear: if worlds
                    .iter()
                    .filter(|world_stats| world_stats.world.has_unlockables())
                    .all(|world_stats| world_stats.a_side.full_clear.is_some())
                {
                    Some(
                        worlds
                            .iter()
                            .filter(|world_stats| world_stats.world.has_unlockables())
                            .map(|world_stats| world_stats.a_side.full_clear.unwrap_or_log())
                            .sum(),
                    )
                } else {
                    None
                },
                heart: worlds.iter().all(|world_stats| world_stats.a_side.heart),
                common: SideStatsCommon {
                    completed: worlds
                        .iter()
                        .filter(|world_stats| world_stats.world.has_unlockables())
                        .all(|world_stats| world_stats.a_side.common.completed),
                    berries: worlds
                        .iter()
                        .filter(|world_stats| world_stats.world.has_unlockables())
                        .flat_map(|world_stats| {
                            (0..world_stats.red_berries()).map(|n| n.to_string()).map(
                                move |mut s| {
                                    s.push(':');
                                    s.push_str(world_stats.world.name());
                                    s
                                },
                            )
                        })
                        .collect(),
                    fewest_dashes: if worlds
                        .iter()
                        .filter(|world_stats| world_stats.world.has_unlockables())
                        .all(|world_stats| world_stats.a_side.common.fewest_dashes.is_some())
                    {
                        Some(
                            worlds
                                .iter()
                                .filter(|world_stats| world_stats.world.has_unlockables())
                                .map(|world_stats| {
                                    world_stats.a_side.common.fewest_dashes.unwrap_or_log()
                                })
                                .sum(),
                        )
                    } else {
                        None
                    },
                    fewest_deaths: if worlds
                        .iter()
                        .filter(|world_stats| world_stats.world.has_unlockables())
                        .all(|world_stats| world_stats.a_side.common.fewest_deaths.is_some())
                    {
                        Some(
                            worlds
                                .iter()
                                .filter(|world_stats| world_stats.world.has_unlockables())
                                .map(|world_stats| {
                                    world_stats.a_side.common.fewest_deaths.unwrap_or_log()
                                })
                                .sum(),
                        )
                    } else {
                        None
                    },
                    single_run: if worlds
                        .iter()
                        .filter(|world_stats| world_stats.world.has_unlockables())
                        .all(|world_stats| world_stats.a_side.common.single_run.is_some())
                    {
                        Some(
                            worlds
                                .iter()
                                .filter(|world_stats| world_stats.world.has_unlockables())
                                .map(|world_stats| {
                                    world_stats.a_side.common.single_run.unwrap_or_log()
                                })
                                .sum(),
                        )
                    } else {
                        None
                    },
                },
            },
            b_side: BSideStats {
                common: SideStatsCommon {
                    completed: worlds
                        .iter()
                        .filter(|world_stats| world_stats.world.has_unlockables())
                        .all(|world_stats| world_stats.b_side.common.completed),
                    berries: BTreeSet::new(),
                    fewest_dashes: if worlds
                        .iter()
                        .filter(|world_stats| world_stats.world.has_unlockables())
                        .all(|world_stats| world_stats.b_side.common.fewest_dashes.is_some())
                    {
                        Some(
                            worlds
                                .iter()
                                .filter(|world_stats| world_stats.world.has_unlockables())
                                .map(|world_stats| {
                                    world_stats.b_side.common.fewest_dashes.unwrap_or_log()
                                })
                                .sum(),
                        )
                    } else {
                        None
                    },
                    fewest_deaths: if worlds
                        .iter()
                        .filter(|world_stats| world_stats.world.has_unlockables())
                        .all(|world_stats| world_stats.b_side.common.fewest_deaths.is_some())
                    {
                        Some(
                            worlds
                                .iter()
                                .filter(|world_stats| world_stats.world.has_unlockables())
                                .map(|world_stats| {
                                    world_stats.b_side.common.fewest_deaths.unwrap_or_log()
                                })
                                .sum(),
                        )
                    } else {
                        None
                    },
                    single_run: if worlds
                        .iter()
                        .filter(|world_stats| world_stats.world.has_unlockables())
                        .all(|world_stats| world_stats.b_side.common.single_run.is_some())
                    {
                        Some(
                            worlds
                                .iter()
                                .filter(|world_stats| world_stats.world.has_unlockables())
                                .map(|world_stats| {
                                    world_stats.b_side.common.single_run.unwrap_or_log()
                                })
                                .sum(),
                        )
                    } else {
                        None
                    },
                },
            },
            c_side: CSideStats {
                common: SideStatsCommon {
                    completed: worlds
                        .iter()
                        .filter(|world_stats| world_stats.world.has_unlockables())
                        .all(|world_stats| world_stats.c_side.common.completed),
                    berries: BTreeSet::new(),
                    fewest_dashes: if worlds
                        .iter()
                        .filter(|world_stats| world_stats.world.has_unlockables())
                        .all(|world_stats| world_stats.c_side.common.fewest_dashes.is_some())
                    {
                        Some(
                            worlds
                                .iter()
                                .filter(|world_stats| world_stats.world.has_unlockables())
                                .map(|world_stats| {
                                    world_stats.c_side.common.fewest_dashes.unwrap_or_log()
                                })
                                .sum(),
                        )
                    } else {
                        None
                    },
                    fewest_deaths: if worlds
                        .iter()
                        .filter(|world_stats| world_stats.world.has_unlockables())
                        .all(|world_stats| world_stats.c_side.common.fewest_deaths.is_some())
                    {
                        Some(
                            worlds
                                .iter()
                                .filter(|world_stats| world_stats.world.has_unlockables())
                                .map(|world_stats| {
                                    world_stats.c_side.common.fewest_deaths.unwrap_or_log()
                                })
                                .sum(),
                        )
                    } else {
                        None
                    },
                    single_run: if worlds
                        .iter()
                        .filter(|world_stats| world_stats.world.has_unlockables())
                        .all(|world_stats| world_stats.c_side.common.single_run.is_some())
                    {
                        Some(
                            worlds
                                .iter()
                                .filter(|world_stats| world_stats.world.has_unlockables())
                                .map(|world_stats| {
                                    world_stats.c_side.common.single_run.unwrap_or_log()
                                })
                                .sum(),
                        )
                    } else {
                        None
                    },
                },
            },
        });

        Self {
            version,
            name,
            gems,
            cheat_mode,
            assist_mode,
            variant_mode,
            total_berries,
            worlds,
        }
    }
}

impl WorldStats {
    pub fn from_save(area_stats: &minidom::Element) -> Self {
        assert!(area_stats.name() == "AreaStats");

        let world = area_stats.expect_parse_attr::<u32>("ID").into();

        let modes = area_stats
            .expect_child("Modes")
            .children()
            .collect::<Vec<_>>();
        assert!(modes.len() == 3);

        let sides_common = modes
            .iter()
            .map(|area_mode_stats| SideStatsCommon::from_save(area_mode_stats))
            .collect::<Vec<_>>();

        let a_side = ASideStats {
            cassette: area_stats.expect_parse_attr::<bool>("Cassette"),
            heart: modes[0].expect_parse_attr::<bool>("HeartGem"),
            full_clear: {
                let decimicroseconds: u64 = modes[0].expect_parse_attr("BestFullClearTime");
                if decimicroseconds == 0 {
                    None
                } else {
                    Some(Duration::from_nanos(decimicroseconds * 100))
                }
            },
            common: sides_common[0].clone(),
        };
        let b_side = BSideStats {
            common: sides_common[1].clone(),
        };
        let c_side = CSideStats {
            common: sides_common[2].clone(),
        };

        Self {
            world,
            a_side,
            b_side,
            c_side,
        }
    }
}

impl SideStatsCommon {
    pub fn from_save(area_mode_stats: &minidom::Element) -> Self {
        assert!(area_mode_stats.name() == "AreaModeStats");

        let completed = area_mode_stats.expect_parse_attr("Completed");

        let single_run;
        let fewest_dashes;
        let fewest_deaths;

        let single_run_completed_attr = area_mode_stats.attr("SingleRunCompleted");
        let single_run_completed = single_run_completed_attr == Some("true");
        if single_run_completed {
            single_run = {
                let decimicroseconds: u64 = area_mode_stats.expect_parse_attr("BestTime");
                Some(Duration::from_nanos(decimicroseconds * 100))
            };
            fewest_dashes = Some(area_mode_stats.expect_parse_attr("BestDashes"));
            fewest_deaths = Some(area_mode_stats.expect_parse_attr("BestDeaths"));
        } else {
            single_run = None;
            fewest_dashes = None;
            fewest_deaths = None;
        };

        let berries = area_mode_stats
            .expect_child("Strawberries")
            .children()
            .map(|entity_id| entity_id.attr("Key").unwrap_or_log().to_string())
            .collect::<BTreeSet<_>>();

        Self {
            completed,
            single_run,
            fewest_dashes,
            fewest_deaths,
            berries,
        }
    }
}
