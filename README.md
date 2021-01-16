# Iron Tank

Iron Tank is a fast and reliable judge written in Rust.

> Judge is a program which is commonly used in XCPC, OI and other programming contest to check if the competitors's solution code is correct or not by supervising the whole running process of it, for example, time and memory usage, and comparing the final result it outputs with the predefined answer.

## Install

Iron Tank relies on the features that only Linux supports at present, which means you can ONLY USE IT ON LINUX.

### Install From Pre-builded

1. Download the newest version from release page, including `iron_tank` and `iron_cell`.
2. Put the executable file you just downloaded in one same folder.
3. Done.

### Install From Source

1. Install Rust toolchain using `Rustup`.
2. Clone the repo.
3. Run command `cargo build --release` in the root directory.
4. Look for path `target/release` to find `iron_tank` and `iron_cell`.
5. Back to the step 2 in **Install From Pre-builded**.

## Usage

## Normal Mode

Iron Tank supports `normal` mode now.

* Input and answer are readed from file.
* Program io use standard io stream.
* Program should only use limited memory and exit in limited time, or it will be killed.
* Program are granted ONLY basic permission such as allocating memory, reading standard stream and some system-related operations.

This is the most common and useful for the most situation.

```bash
$ iron_tank normal <exec> -i <input> -a <answer> -t <time-limit> -m <memory-limit> -c <compare-mode>
```

* `<exec>`, the path of program to be run.
* `<input>`, the input file for program.
* `<answer>`, the answer file.
* `<time-limit>`, time limit(MS) for program.
* `<memory-limit>`, memory limit(MB) for program.
* `<compare-mode>`, define the approach to compare the output and answer.

Just for example. The command below will start a container, in which program can only use 256 MB memory at most, run no longer than 1 second, only read/write to standard io without permissions such as opening file, conencting network and forking new process. The output by `./user_code` is compared with content in `1.ans` line by line.

```
$ iron_tank normal ./user_code -i 1.in -a 1.ans -t 1 -m 256 -c line
```

### Comparation Mode

`full`. Output must be the absolutely same with Answer, including blank characters.

They are different.

```
a b

```

```
a b
```

`line`. Output and Answer are trimmed firstly to remove the blank chars at the begin and end position of them. Then comparation are held on each line of them, ignoring blank chars at the end position.

They are the same.

```
a b
d
```

```

a b   
d

```

`value`. Output and Answer are compared without any blank chars.

They are the same.

```
a b
d
```

```

ab d

```

Attention that status `PE` may appear when comparation mode is set to the first or second one.