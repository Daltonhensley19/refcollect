
# Refcollect

Refcollect is a Rust library for managing references to objects via roots. These roots (and their associated references) are managed internally via the `MarkandSweepGC` data structure. This data structure is intended to be used as a simple garbage collector that implements the so-called [mark and sweep algorithm](https://www.cs.odu.edu/~zeil/cs330/f13/Public/garbageCollection/garbageCollection-htmlsu5.html).

## Table of Contents

- [Installation](#installation)
- [License](#license)
- [Contact](#contact)

## Installation

Ensure you have the following prerequisites installed on your system:

- [Git](https://git-scm.com/)
- [Rust](https://www.rust-lang.org/)

### Clone the Repository

First, clone the repository:
```bash
git clone https://github.com/Daltonhensley19/refcollect.git
cd refcollect
```

### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build and Run Refcollect
```bash
cargo run 
```

## License
Distributed under the MIT License.

## Contact

For any inquiries or feedback, feel free to reach out:

    Email: dzhensley@moreheadstate.edu
    GitHub Issues: https://github.com/Daltonhensley19/refcollect/issues
