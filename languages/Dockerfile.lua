FROM nickblah/lua:5.3-bullseye

RUN apt-get update && apt-get install -y dos2unix wget build-essential unzip

RUN wget https://luarocks.org/releases/luarocks-3.8.0.tar.gz && \
    tar zxpf luarocks-3.8.0.tar.gz && \
    cd luarocks-3.8.0 && \
    ./configure && \
    make && \
    make install && \
    cd .. && \
    rm -rf luarocks-3.8.0 luarocks-3.8.0.tar.gz

RUN luarocks install lua-csv
RUN luarocks install ljsyscall
RUN luarocks install lua-imagick

RUN useradd -m -s /bin/bash executor && mkdir -p /home/executor/sandbox && chown -R executor:executor /home/executor/sandbox

WORKDIR /home/executor/sandbox
COPY executor_script.sh .

RUN dos2unix executor_script.sh && chmod +x executor_script.sh

USER executor

CMD ["./executor_script.sh"]
