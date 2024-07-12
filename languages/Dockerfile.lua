FROM nickblah/lua:5.3-bullseye

RUN apt-get update && apt-get install -y dos2unix && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash executor && mkdir -p /home/executor/sandbox && chown -R executor:executor /home/executor/sandbox

WORKDIR /usr/src/executor
COPY executor_script.sh .

RUN dos2unix executor_script.sh && chmod +x executor_script.sh

USER executor

CMD ["./executor_script.sh"]
