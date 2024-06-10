+++
title = "Building Openpose"
description = """
"""
date = 2024-06-10
draft = true

[taxonomies]
tags = []
[extra]
toc = true
+++

Steps to make CPU only Openpose on Ubuntu 22.04, set up with LambdaStack.

```
git clone https://github.com/CMU-Perceptual-Computing-Lab/openpose

sudo apt-get install cmake-qt-gui libopencv-dev protobuf-compiler libgoogle-glog-dev libboost-all-dev libhdf5-dev libatlas-base-dev install python3-dev

sudo pip3 install numpy opencv-python

sudo bash ./scripts/ubuntu/install_deps.sh

mkdir build

cd build

cmake-gui ..
```

Press the Configure button, set generator to Unix Makefiles. Hit finish.

Enable the BUILD_PYTHON flag and click Configure again.

Set the GPU_MODE flag to CPU_ONLY and click Configure again.

It might fail again with downloads failed. Just hit configure again and there should be no errors.

If there are no errors, click Generate and exit cmake-gui.

```
make -j`nproc`
```

If the build fails with

```
~/openpose/3rdparty/caffe/src/caffe/util/io.cpp: In function ‘bool caffe::ReadProtoFromBinaryFile(const char*, google::protobuf::Message*)’:
~/openpose/3rdparty/caffe/src/caffe/util/io.cpp:57:34: error: no matching function for call to ‘google::protobuf::io::CodedInputStream::SetTotalBytesLimit(const int&, int)’
   57 |   coded_input->SetTotalBytesLimit(kProtoReadBytesLimit, 536870912);
      |   ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
In file included from ~/openpose/3rdparty/caffe/src/caffe/util/io.cpp:2:
/usr/include/google/protobuf/io/coded_stream.h:384:8: note: candidate: ‘void google::protobuf::io::CodedInputStream::SetTotalBytesLimit(int)’
  384 |   void SetTotalBytesLimit(int total_bytes_limit);
      |        ^~~~~~~~~~~~~~~~~~
/usr/include/google/protobuf/io/coded_stream.h:384:8: note:   candidate expects 1 argument, 2 provided
```

then edit line 57 of `~/openpose/3rdparty/caffe/src/caffe/util/io.cpp` to remove the first argument.

```
make -j`nproc`
```

Build should succeed.
