FROM eclipse-temurin:21-jre
LABEL authors="jesper"
WORKDIR /minecraft

RUN curl -O https://piston-data.mojang.com/v1/objects/8dd1a28015f51b1803213892b50b7b4fc76e594d/server.jar
COPY eula.txt .
COPY server.properties .

EXPOSE 25565/tcp
ENTRYPOINT ["java", "-Xmx1024M", "-Xms1024M", "-jar", "server.jar", "nogui"]
