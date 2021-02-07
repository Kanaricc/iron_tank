# Iron Tank

**WIP. The usage and data format may change dramatically as development progresses.**

[中文](./README.zh.md)

Iron Tank is a fast and reliable judge written in Rust.

> Judge is a program used commonly in ICPC, CCPC, OI and other programming contests to check if the competitors' solution code is correct or not by supervising the whole running process, for example, time and memory usage, and comparing the final result it outputs with the predefined answer.

## Install

Iron Tank relies on the features that Linux supports at present. I have only tested it on Linux. Not sure will it work on OS X, FreeBSD or not.

Does not support Windows.

### Install From Pre-builded Version

1. Download the newest version from the release page,  `iron_tank` and `iron_cell`.
2. Put the executable file you just downloaded in one same folder.
3. Done.

### Install From Source

1. Install Rust toolchain using `Rustup`.
2. Clone the repo.
3. Run command `cargo build --release` in the root directory.
4. Look for path `target/release` to find `iron_tank` and `iron_cell`.
5. Jump to the step 2 in **Install From Pre-builded Version**.

## Usage

### Server

By starting Iron Tank in server mode, you get a judge backend. (WIP)

### Normal

* Input and answer are read from file.
* Program IO uses standard io stream.
* Program should only use limited memory and exit in limited time, or it will be killed.
* Program is granted ONLY basic permissions such as allocating memory, reading standard stream and some system-related operations.

This is the common and useful mode for most situation.

Command pattern:

```bash
$ iron_tank normal <exec> -i <input> -a <answer> -t <time-limit> -m <memory-limit> -c <compare-mode>
```

* `<exec>`, the path of program to be run.
* `<input>`, the input file for program.
* `<answer>`, the answer file.
* `<time-limit>`, time limit(MS) for program.
* `<memory-limit>`, memory limit(MB) for program.
* `<compare-mode>`, define the approach to compare the output and answer.

Just for example. The command below will start a "cell", in which program can only use *about* 256 MB memory at most, run no longer than *about* 1 second, only read/write to standard io without permissions such as opening file, conencting network and forking new process. The output by `./user_code` is compared with content of `1.ans` line by line.

```
$ iron_tank normal ./user_code -i 1.in -a 1.ans -t 1 -m 256 -c line
```

#### Judge Result

**(WIP)**

10 kinds of result are provided for now.

```rust
pub enum JudgeStatus {
    Uncertain,
    Accept,
    WrongAnswer,
    PresentationError,
    MemoryLimitExceeded,
    TimeLimitExceeded,
    InteractionTimeLimitExceeded,
    ComplierError,
    ComplierLimitExceeded,
    RuntimeError,
}
```

#### Comparation Mode

* `full`. Output must be the absolutely same with Answer, including blank characters.

They are the same.

```
I Can EatGlass

```

```
I Can EatGlass

```

* `line`. Output and Answer are trimmed firstly to remove the blank chars at the beginning and ending position of them. Then comparison are held on each line of them, ignoring blank chars at the ending position. (Output are readed from left to right.)

They are the same. Attention that there are some blank chars after `b` in latter one, and the empty space between `d` and `我能` cannot be ignored.

```
a b
d

我能
吞下玻璃而不伤身体
```

```

a b   
d

我能
吞下玻璃而不伤身体

```

* `value`. Output and Answer are compared without any blank chars.

They are the same.

```
PHP
is
the best language
```

```
PHPisthebest
language
```

Status `PE` may appear when comparison mode is set to the first or second one.

### Speical (Speical Judge)

* Input is readed from file.
* A user-defined checker is used to check if program gives correct output.

This mode is used when

* There are many possible correct answers.
* Output should be checked in real-time.
* Other situation that normal mode cannot fit.

Command pattern:

```bash
$ iron_tank special <exec> -i <input> -c <checker> -t <time-limit> -m <memory-limit>
```

* `<exec>`, the path of program to be run.
* `<input>`, the input file for program.
* `<checker>`, the path of checker
* `<time-limit>`, time limit(MS) for program.
* `<memory-limit>`, memory limit(MB) for program.

#### Checker

A checker will receive input, output of the program, and give the result of comparison.

Two arguments are provided for the checker passing by `argv`:

* ~~source code file~~
* input file, which is the same as the one for program.
* output file, containing the output of program.

Checker should give output in pattern:

```
<result>
<msg>
```

* `<result>`: same -> Accepted, different -> WrongAnswer, presentation_different -> PresentationError.
* `<msg>`: whatever you want.

MLE, TLE, RE, and other kinds of status are still given by Iron Tank.

For now, make sure your checker is fully tested, as Iron Tank has not run it in container, which means checker's crashing downs the whole judge process too.

A checker sample:

```cpp
#include <iostream>
#include <fstream>
using namespace std;

int main(int argc,char* argv[]){
    ifstream input(argv[1]);
    ifstream output(argv[2]);

    string s1, s2;
    input >> s1;
    output >> s2;

    if(s1 == s2) {
        cout << "same" << endl << "" << endl;
    } else {
        cout << "different" << endl << "" << endl;
    }

    return 0;
}
```

### Interactive

* Input is dynamically generated by a program called `interactor` **on-the-fly**.
* User program uses standard IO.
* Output is checked by `interactor` **on-the-fly**.

In short, `interactor` and user program are *directly* connected, they can interact in real-time.

This mode is used when

* The next input comes from previous output.
* You want to use the most powerful strategy to make things difficult for the user based on its output.(Oh, you are so bad~)
* (You'll find a way to use it.)

Command pattern:

```bash
$ iron_tank special <interactor> <exec> -i <input> -t <time-limit> -m <memory-limit>
```

#### Interactor

An `interactor` is a program, output and input of which will be connected to the input and output of user program.

`interactor` should put the result in stderr.

```
<result>
[msg]
```

* `<result>`: same -> Accepted, different -> WrongAnswer, presentation_different -> PresentationError.
* `[msg]`: whatever you want. It is ok to ignore it.

For example,

```cpp
#include <iostream>
using namespace std;

int main(){
    bool ok=true;
    for(int i=0;i<10;i++){
        cout<<i<<endl;
        int x;cin>>x;
        fout<<x<<endl;
        if(x!=(1<<i))ok=false;
    }

    if(ok){
        cerr<<"same"<<endl;
    }else{
        cerr<<"different"<<endl;
    }

    return 0;
}
```

**MAKE SURE the interactor always flushes its IO buffers!** It is also important to notice users about that.


### Prefab

By using a YAML configuration file, you can edit a problem beforehand and quickly use that configuration to create tasks.

Command pattern:

```bash
$ iron_tank prefab <config> <exec>
```

* `<config>`: config file.
* `<exec>`: the path of program to be run.

To make a prefab,

1. Create a folder, named by the title of problem (for example, `A`) or whatever you want.
2. Touch a new file in it named `problem.yaml`.

Content of a `problem.yaml` likes

```yaml
name: A                     # the title of problem
limit_config:
  time_limit: 1000          # time limit (ms)
  memory_limit: 256         # and memory limit (MB)
judge_mode:                 # judge mode
  Normal:                   # here we use normal mode
    comparision_mode: Line  # compare output using `Line` mode
cases:                      # you can add many cases for one problem
  - inputfile_path: 1.in    # the path is relative to this config file
    answerfile_path: 1.ans
  - inputfile_path: 2.in
    answerfile_path: 2.ans
```

Then, prepare and put your data in correct place according to this config file. I suggest you put them in the same folder.

## Details

### Time and Memory Limits

In fact, the real limits of time and memory are *two times* higher than values you set. That means a program can still run and exit normally even it has allocated more memory and used more cpu time than limit you set.

Iron Tank will give `TLE` when the time usage is longer than limit.

Will give `MLE` when

* Program is killed by cell, and the peak memory usage overflow.
* Program is killed by cell, and it exits with error message caused by memory allocation.
* Program exits normally, but the peak memory usage overflow.

There is a possible existing problem that a program has not touched the limit unless the next allocation in future were done. Once such a allocation is put up with, program will be killed immediately. Since vary languages and compilers act differently, this situation has not been all covered now. That means a program may be killed, leaving result to be *Runtime Error* while it is actually *Memory Limit Exceeded*.

> I have encountered this problem on some Online Judge platforms (won't specify them here). Hope it can be solved by the development of this repo.

### Data Format

**Be careful for data format.** Error caused by *invalid* data is hard to be observed. Simplely making mistakes in config just let Judge exits with error, while an invalid input leads to wrong judge result leaving everything seems to be no problems.

* **Use ASCII or UTF-8 for all data, including file and checker's output.**
* **Input should ends with a new empty line, unless you know what you are doing.** For C/C++, `scanf()` and `cin` only take input at the moment when a `enter` is entered. Missing such thing will let the program wait for it till it is killed because of TLE. But some languages does not care about that such as Python. If the input format is important for your problem, you may ignore this and mention it to users.
* **Fully test your data.** Though it is none of bussiness of Judge.

### A program luckily uses both too much time and memory...

`TLE` is concerned first.