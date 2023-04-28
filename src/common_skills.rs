// import hasmap
use phf::phf_map;

pub static SKILLS: phf::Map<&'static str, &'static str> = phf_map! {
    "0 o" => "Tuck Jump",
    "0 <" => "Pike Jump",
    "0 /" => "Straddle Jump",
    "0 o f" => "Tuck Jump",
    "0 < f" => "Pike Jump",
    "0 / f" => "Straddle Jump",
    "41 o f" => "Tuck Barani",
    "41 < f" => "Pike Barani",
    "41 / f" => "Straight Barani",
    "40 o f" => "Tuck Front",
    "40 < f" => "Pike Front",
    "40 / f" => "Straight Front",
    "42 /" => "Full Twist Back",
    "44 /" => "Double Twist Back",
    "801 o f" => "Tuck Half Out",
    "801 < f" => "Pike Half Out",
    "801 / f" => "Half Out Layout",
};