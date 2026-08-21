# ACMake

Advanced CMake - A command-line tool to manage CMake projects, almost like Rust's cargo!

## Geting started

Install acmake using cargo:
```
cargo install acmake
```

### Creating new projects

Create a new CMake C++ binary project:
```
acmake new -l c++ my-new-project
```
This will create the following tree structure:
```
├── my-new-project
│   ├── CMakeLists.txt
│   └── main.cpp
```
If you want to customize the root folder name, pass the `-f` flag:
```
acmake new -l c++ -f our-new-project my-new-project
```
And the output will be:
```
├── our-new-project
│   ├── CMakeLists.txt
│   └── main.cpp
```
Note: the `-f` flag only affects the folder name.
The project name inside CMakeLists is the last argument passed to `acmake`.
In this example, it will be `my-new-project`.

### Program parameters

#### Language
The `-l` flag specifies the language.
You can create a C++ project using any of the following language flags: `-l c++`, `-l cpp`, `-l cxx`.
To create a C project, use `-l c`.

#### Language standard version
By default, ACMake generates C++17 and C11 projects.
In order to specify the language standard, append a colon to the passed `-l` language, followed by the standard version.
For example, the following command creates a C++23 project:
```
acmake new -l cpp:23 my-new-project
```