FROM nickblah/lua:5.3-bullseye

RUN apt-get update && apt-get install -y dos2unix wget build-essential unzip git cmake libmagickwand-dev ca-certificates lua5.3 liblua5.3-dev --no-install-recommends

RUN wget http://luajit.org/download/LuaJIT-2.0.5.tar.gz && \
    tar zxpf LuaJIT-2.0.5.tar.gz && \
    cd LuaJIT-2.0.5 && \
    make && \
    make install && \
    cd .. && \
    rm -rf LuaJIT-2.0.5 LuaJIT-2.0.5.tar.gz

RUN wget https://luarocks.org/releases/luarocks-3.9.2.tar.gz && \
    tar zxpf luarocks-3.9.2.tar.gz && \
    cd luarocks-3.9.2 && \
    ./configure --lua-version=5.3 && \
    make && \
    make install && \
    cd .. && \
    rm -rf luarocks-3.9.2 luarocks-3.9.2.tar.gz

RUN git clone https://github.com/isage/lua-imagick.git /tmp/lua-imagick && \
    cd /tmp/lua-imagick && \
    mkdir build && \
    cd build && \
    cmake .. && \
    make && \
    make install

RUN luarocks install lua-csv

RUN useradd -m -s /bin/bash executor && mkdir -p /home/executor/sandbox && chown -R executor:executor /home/executor/sandbox

WORKDIR /usr/src/executor
COPY executor_script.sh .

RUN dos2unix executor_script.sh && chmod +x executor_script.sh

USER executor

CMD ["./executor_script.sh"]
