extern crate reqwest;

pub mod remote_tarball;

use remote_tarball::RemoteTarball;

#[cfg(test)]
const NODE: &'static str = "https://nodejs.org/dist/v6.15.1/node-v6.15.1-darwin-x64.tar.gz";

const YARN: &'static str = "https://github.com/yarnpkg/yarn/releases/download/v1.12.3/yarn-v1.12.3.tar.gz";

fn main() {
    let tarball = RemoteTarball::fetch(YARN).unwrap();

    println!("fetching: {}", YARN);
    println!("compressed size: {}", tarball.compressed_size());
    println!("uncompressed size: {}", tarball.uncompressed_size().unwrap().unwrap());
}

#[test]
fn test_node() {
    let tarball = RemoteTarball::fetch(NODE).unwrap();
    assert_eq!(tarball.compressed_size(), 12294157);
    assert_eq!(tarball.uncompressed_size().unwrap(), Some(46173184));
}

#[test]
fn test_yarn() {
    let tarball = RemoteTarball::fetch(YARN).unwrap();
    assert_eq!(tarball.compressed_size(), 1166553);
    assert_eq!(tarball.uncompressed_size().unwrap(), Some(4904960));
}
