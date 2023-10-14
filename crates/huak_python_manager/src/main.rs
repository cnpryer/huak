use huak_home::huak_home_dir;
mod releases;

fn main() {
    println!("{:?}", huak_home_dir());
    println!("{:?}", releases::RELEASES[0].url);
}
