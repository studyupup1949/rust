import os

flags = [
    '-Wall',
    '-Wextra',
    '-x',
    'c++',
    '-std=c++14',
]

include_dirs = [
    'include',
    'build/include',
    'build/target',
]

for dir in include_dirs:
    flags.append('-I')
    flags.append(os.path.abspath(dir))

def FlagsForFile(filename):
    return {
        'flags': flags,
        'do_cache': True
    }
