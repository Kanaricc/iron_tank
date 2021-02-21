# Iron Tank (WIP)

![Rust](https://github.com/Kanaricc/iron_tank/workflows/Rust/badge.svg)

[English](./README.md)

Iron Tank （期望）是一个使用 Rust 编写的快速且稳定的评测器。下称 `tank`。

> 所谓评测器，是一种常用于 ICPC、CCPC 等程序设计竞赛中的特殊程序，用于检查比赛选手的代码是否正确（通常情况下将代码视作黑箱，给予输入并对比输出与答案是否相符），并限制和记录代码运行全程中的时间、内存使用情况。

## 安装

目前，`tank` 依赖于 Linux 的一些特性。我并不清楚它能不能在 OS X 或者 FreeBSD 等系统上正常运行。

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
 
## 服务模式

启动服务模式，在该模式下你将得到一个功能完备的评测后端。（WIP）

## 快捷使用

### 标准评测模式

* 输入和答案通过文件给出。
* 被评测代码从标准 IO 中读取和输出数据。
* 被评测代码必须在受限制的内存和时间内得到结果，否则将被杀死。
* 被评测代码只能使用最基本的权限，例如申请内存、读取标准流以及其他必要的基本操作。读写额外文件、网络连接等非正常操作均被禁止。

这是最普通且常用的模式。

命令格式：

```bash
$ tank_cli normal <src> -i <input> -a <answer> -t <time-limit> -m <memory-limit> -c <compare-mode>
```

* `<src>`, 将被运行的用户代码。
* `<input>`, 样例输入文件。
* `<answer>`, 样例答案文件。
* `<time-limit>`, 时间限制。（MS）
* `<memory-limit>`, 内存限制。（MB）
* `<compare-mode>`, 输出的比较模式。

举个例子。下方的命令将会启动一个评测单元，程序只能使用最多*大约* 256MB 的内存，且必须在*大约* 1秒内得到结果并正常退出。程序只能操作标准 IO，无法自行读写任何文件、连接网络、创建线程等。程序输出到标准 IO 的输出将会和 `1.ans` 按照**行模式**进行对比。


```
$ tank_cli normal ./user_code -i 1.in -a 1.ans -t 1 -m 256 -c line
```

#### 评测结果

**(WIP)**

现有 10 种可能的结果。

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
* `line`。首先输出和答案将先被移除开头和结尾的所有空白字符，此后将一一对比二者的每一行内容是否相同，每行最后的空白字符将被忽略。（阅读习惯为从左到右）
* `value`。输出和答案将被移除所有空白符后再比较。

对于前两种比较方法，评测器可能给出 **PE** 的结果。

### Speical 模式 (Speical Judge)

* 输入由文件给出。
* 用户定义一个程序，用于判断用户代码是否给出正确答案。
* 其他与标准模式一致。

命令格式：

```bash
$ tank_cli special <checker> <src> -i <input> -t <time-limit> -m <memory-limit>
```

* `<src>`，用户代码。
* `<input>`，输入文件。
* `<checker>`，checker 程序。
* `<time-limit>`，参照标准模式。
* `<memory-limit>`，参照标准模式。

#### Checker

Checker 应该接收提供给用户代码的输入、来自用户代码的输出，并且给出检查结果。

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

### Interactive

* 由 `interactor` 动态产生输入。
* 用户使用标准 IO 读取和输出。
* `interactor` 即时检查输出。

一句话说，`interactor` 和用户程序被直接联系在一起，它们能够即时地对对方的动作作出反应。

命令格式：

```bash
$ tank_cli special <interactor> <src> -i <input> -t <time-limit> -m <memory-limit>
```

#### Interactor

`interactor`（交互器） 是一个特殊的程序，它的输入（stdin）和输出（stdout）会被「直接」和用户程序连接。

`interactor` 通过标准错误流（stderr）和评测器沟通。你应该输出

一个 interactor 的例子。

```cpp
#include <iostream>
using namespace std;

int main(){
    bool ok=true;
    for(int i=0;i<10;i++){
        cout<<i<<endl;
        int x;cin>>x;
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

## 问题项目

通过使用 YAML 文件，你可以预设一个问题项目。预计这将是 tank 中**问题**的**项目形式**。

1. 创建一个文件夹，名字是问题的标题，保证这个标题和 YAML 里的一致。例如这里，我们设为A。
2. 在目录下创建 `problem.yaml`。

`problem.yaml` 的内容形如

```yaml
name: A                       # 问题标题
limitConfig:
  time:imit: 1000             # 时间限制 (ms)
  memory:imit: 256            # 内存限制 (MB)
judgeMode:                    # 评测模式
  Normal:                     # 这里使用了普通模式
    comparisionMode: Line     # 使用按行比较模式
inputLint:                    # 添加检查器来检查你的数据是否正确
  linters:
    - unexpected-bytes
    - consecutive-empty-lines
    - start-with-empty-line
    - extra-spaces-after-lines
    - consecutive-spaces
  customLints:
    - |-
      data.rint();
      data.eeof();
      0
answerLint:
  linters:
    - unexpected-bytes
    - consecutive-empty-lines
    - start-with-empty-line
    - extra-spaces-after-lines
    - consecutive-spaces
  customLints:
    - |-
      data.rint();
      data.eeof();
      0
cases:                        # 为问题准备的测试点
  - inputfilePath: 1.in       # 目录应相对于本设置文件给出
    answerfilePath: 1.ans
  - inputfilePath: 2.in
    answerfilePath: 2.ans
```

`tank` 为问题项目提供

* 数据的规范与检查，用于防止数据出现意外失误。
* 问题的所有预设参数，你无需在评测时再次指定限制。
* 多组数据，更加符合一般的评测需求。

## Lint

通过问题项目，你可以设置对问题数据的 lint 规则。

此后可以使用命令

```bash
$ tank_cli lint <config>
```

来执行对某个项目的检查。

## 语言支持

| 编译器  | 命令                          |
| ------- | ----------------------------- |
| g++     | `g++ <input> -o <output> -O2` |
| Python3 | `python3 <src>`               |