# This script takes care of building your crate and packaging it for release

set -ex

main() {
    cd rustual-boy-cli

    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    # TODO Update this to build the artifacts that matter to you
    cargo rustc --bin rustual-boy-cli --release -- -C lto

    # TODO Update this to package the right artifacts
    cp target/release/rustual-boy-cli $stage/
    cp doc/* $stage/
    cp LICENSE-APACHE $stage/
    cp LICENSE-MIT $stage/
    cp LICENSE-THIRD-PARTY $stage/

    cd $stage
    tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
    cd $src

    rm -rf $stage
}

main
