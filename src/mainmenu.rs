pub(crate) enum Ship {
    Blank,
    Spear,
    Paladin,
}
impl Ship {
    const ALL: [Self; 3] = [Self::Blank, Self::Spear, Self::Paladin];
    const fn name(&self) -> &'static str {
        match self {
            Self::Blank => "Blank",
            Self::Spear => "Spear",
            Self::Paladin => "Paladin",
        }
    }
    const fn description(&self) -> &'static str {
        match self {
            Self::Blank => {
                "Good all around. Has \
                the power to banish \
                bullets in a blink"
            }
            Self::Spear => {
                "A very powerfull ship \
                favors offense at the \
                expense of defense"
            }
            Self::Paladin => {
                "This ship was built to \
                last. Has the ability \
                to convert bullets"
            }
        }
    }
}
pub(crate) enum MainEntry {
    Start,
    ShipSelect,
}
impl MainEntry {
    const ALL: [Self; 2] = [Self::Start, Self::ShipSelect];
    const fn text(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::ShipSelect => "select ship",
        }
    }
}
pub(crate) enum Submenu {
    Main(MainEntry),
    ShipSelect { highlight: Ship },
}
pub(crate) struct Mainmenu {
    pub(crate) ship: Ship,
    pub(crate) menu: Submenu,
}
impl Default for Mainmenu {
    fn default() -> Self {
        Self {
            ship: Ship::Blank,
            menu: Submenu::Main(MainEntry::Start),
        }
    }
}

fn init_main() {}
fn update_main() {}
