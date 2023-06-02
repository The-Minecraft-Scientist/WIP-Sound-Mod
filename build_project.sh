#!/bin/bash
oldhome=JAVA_HOME
JAVA_HOME=/Library/Java/JavaVirtualMachines/temurin-18.jdk/Contents/Home/
export JAVA_HOME
./gradlew :rust:native-jni:clean

./gradlew :fabric:build

 ./gradlew :fabric:configureClientLaunch
 JAVA_HOME=oldhome
 export JAVA_HOME
