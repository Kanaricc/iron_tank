# Iron Tank (WIP)

[English](./README.md)

Iron Tank （期望）是一个使用 Rust 编写的快速且稳定的评测器。

> 所谓评测器，是一种常用于 ICPC、CCPC 等程序设计竞赛中的特殊程序，用于检查比赛选手的代码是否正确（通常情况下将代码视作黑箱，给予输入并对比输出与答案是否相符），并限制和记录代码运行全程中的时间、内存使用情况。

## 安装

目前，Iron Tank 依赖于 Linux 的一些特性。我并不清楚它能不能在 OS X 或者 FreeBSD 等系统上正常运行。

**暂不支持 Windows**。

### 从预编译版本安装

1. 从发布页面下载最新的 `iron_tank` 以及 `iron_cell`。
2. 将二者放在同一个文件夹下。
3. 如果必要的话，配置你所需要的 `PATH` 环境。
4. 完成。

### 从源码安装

1. 安装 Rust 工具链管理工具 Rustup。
2. Clone 这个项目。
3. 在项目下运行命令 `cargo build --release`。
4. 从目录 `target/release` 下寻找 `iron_tank` 以及 `iron_cell`。
5. 回到**从预编译版本安装**的第 2 步。

## 使用

### 服务模式

启动服务模式，在该模式下你将得到一个功能完备的评测后端。（WIP）

### 标准评测模式

* 输入和答案通过文件给出。
* 被评测代码从标准 IO 中读取和输出数据。
* 被评测代码必须在受限制的内存和时间内得到结果，否则将被杀死。
* 被评测代码只能使用最基本的权限，例如申请内存、读取标准流以及其他必要的基本操作。读写额外文件、网络连接等非正常操作均被禁止。

这是最普通且常用的模式。

命令格式：

```bash
$ iron_tank normal <exec> -i <input> -a <answer> -t <time-limit> -m <memory-limit> -c <compare-mode>
```

* `<exec>`, 将被运行的用户程序。（目前，你需要事先编译）
* `<input>`, 样例输入文件。
* `<answer>`, 样例答案文件。
* `<time-limit>`, 时间限制。（MS）
* `<memory-limit>`, 内存限制。（MB）
* `<compare-mode>`, 输出的比较模式。

举个例子。下方的命令将会启动一个评测单元，程序只能使用最多*大约* 256MB 的内存，且必须在*大约* 1秒内得到结果并正常退出。程序只能操作标准 IO，无法自行读写任何文件、连接网络、创建线程等。程序输出到标准 IO 的输出将会和 `1.ans` 按照**行模式**进行对比。


```
$ iron_tank normal ./user_code -i 1.in -a 1.ans -t 1 -m 256 -c line
```

#### 评测结果

**(WIP)**

现有 8 种可能的结果。

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

#### 比较方法

* `full`。输出必须和答案完全一致，包括回车、空格、制表符等空白字符。

下方的例子被认为是一样的。

```
I Can EatGlass

```

```
I Can EatGlass

```

* `line`。首先输出和答案将先被移除开头和结尾的所有空白字符，此后将一一对比二者的每一行内容是否相同，每行最后的空白字符将被忽略。（阅读习惯为从左到右）

下方的例子被认为是一样的。注意，在第二个的字母 `b` 之后有着一些空白字符，而且 `d` 和 `我能` 之间的空行不会被忽略。


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

* `value`。输出和答案将被移除所有空白符后再比较。

下方例子被认为是一样的。

```
PHP
is
the best language
```

```
PHPisthebest
language
```

对于前两种比较方法，评测器可能给出 **PE** 的结果。

### Speical 模式 (Speical Judge)

* 输入由文件给出。
* 用户定义一个程序，用于判断用户代码是否给出正确答案。
* 其他与标准模式一致。

你可能会在下列情况下使用这种模式

* 答案不唯一。
* 必须以输出作为输入来主动检查结果是否正确。
* 其他标准模式无法事先的情况。
* （这不是交互模式）

命令格式：

```bash
$ iron_tank special <exec> -i <input> -c <checker> -t <time-limit> -m <memory-limit>
```

* `<exec>`，用户代码。
* `<input>`，输入文件。
* `<checker>`，checker 程序。
* `<time-limit>`，参照标准模式。
* `<memory-limit>`，参照标准模式。

#### Checker

Checker 应该接收提供给用户代码的输入、来自用户代码的输出，并且给出检查结果。

你的 Checker 会在 `argv`（对于 C 系程序来讲）种接收到 2 个参数。

* ~~用户代码~~
* 输入文件位置，内容与用户程序的输入一致。
* 输出文件位置，内容为用户程序的输出。

Checker 必须给出下列格式的输出。

```
<result>
<msg>
```

* `<result>`: `same` -> Accepted, `different` -> WrongAnswer, `presentation_different` -> PresentationError.
* `<msg>`: 你想输出的提示。

注意，MLE、TLE、RE以及其他的结果仍然由评测器给出。

目前，请确保你的 Checker 完全可以信任，评测器还没有将 Checker 也放入容器运行。

下方是一个 checker 的例子。

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

## 细节

### 时间和内存限制


实际上，用户程序所受到的真正时间、内存限制是大于你给出的设置限制的，我们称它为真实限制。这意味着用户程序在使用了稍微多一点的内存和时间后，仍然不会被杀死。不过，这不会影响最终的评测结果。

评测器在用户程序使用超出限制的时间时，总会给出 TLE 的结果。

评测器在下列情况下会给出 MLE 的结果。

* 用户程序被评测单元杀死，并且内存使用峰值超过了限制。
* 用户程序被评测单元杀死，并且附带的错误信息与内存申请失败相关。
* 用户程序正常退出，但是内存使用峰值超过了限制。


这里可能有一个潜在的问题。当一个程序还没有碰到设置限制，但是它的下一次内存申请会让它直接碰到真实限制。这种情况下，一旦程序做出了这种内存申请，它就会被立刻杀死，并留下一些类似于 `bad alloc` 的内存申请失败的错误信息。然而它的内存峰值并没有超过设置限制，所以*很有可能*被评测器认为是运行时错误。有些 OJ 的评测器也有这种问题。

### 数据格式


**注意你的样例数据格式**。格式不正确的数据引起的错误很难被发现，而且容易造成难以挽回的后果。Iron Tank 计划在今后引入数据格式检查。不过任何时候你都该注意：

* **总是使用 ASCII 或者 UTF-8，包括 checker 给出的输出**。
* **输出必须以新的空行结束，除非你知道你在做什么**。对于比赛常用的 C/C++ 来讲，`scanf()` 和 `cin` 只会在输入回车时才真的读入数据（对于选手来讲）。不以空行结束，会让用户程序一直等待，最终导致超时。但是有些语言可能就不会在意这一点。如果输入格式对你的问题很重要，你可以忽略这一点。

### 如果有一个程序同时超时而且爆了内存…

结果会被认为是 TLE。