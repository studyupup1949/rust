FROM ubuntu:14.04

RUN apt-get update
RUN apt-get install -y libcurl4-openssl-dev libelf-dev libdw-dev cmake gcc wget tar
RUN apt-get install -y g++ python2.7
RUN ln -s /usr/bin/python2.7 /usr/bin/python2
RUN apt-get install -y binutils-dev
WORKDIR /root
RUN wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz
RUN tar xzf master.tar.gz
RUN mkdir -p /root/kcov-master/build
WORKDIR /root/kcov-master/build
RUN cmake ..
RUN make
RUN sudo make install
