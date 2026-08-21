# Activity Log

## Introduction

**Activity Log** is a tool that registers a user's work activities into a log history.

Inspired by Git, the main idea is to use CLI commands to execute different actions, such as :
- *commit* a task, wich saves it temporary
- *save* a collection of the last commited tasks
- specifying a topic/subject you are working on (e.g.: do task A for project X)
- ...

### Notice

This is a first experimental version. Please understand that not every details of the tool are documented and there may be bugs to be found and fixed lately.

If so, please request an issue into the project. It would be a great help!

## Commands

```sh
activitylog commit <message> [--section <section-name>]
activitylog save
activitylog switch <subject-name>
activitylog subject <list|add|remove|update>
activitylog convert <output-format>
activitylog generate
```

## History commit format

```log
YYYY-MM-DD hh:mm:ss | Commit message
YYYY-MM-DD hh:mm:ss | Commit message | section name
```