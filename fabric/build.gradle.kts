
plugins {
    id("fabric-loom") version "1.1-SNAPSHOT"
    id("fr.stardustenterprises.rust.importer") version "3.2.5"
}
base {
    val archivesBaseName: String by project
    archivesName.set(archivesBaseName)
}

loom {
    accessWidenerPath.set(file("src/main/resources/soundmod.accesswidener"))
    runs {
        configureEach {
            this.isIdeConfigGenerated = true
        }
    }
}

val modVersion: String by project
version = modVersion
val mavenGroup: String by project
group = mavenGroup

dependencies {
    val minecraftVersion: String by project
    val yarnMappings: String by project
    val loaderVersion: String by project
    val fabricVersion: String by project

    // To change the versions see the gradle.properties file
    minecraft("com.mojang:minecraft:$minecraftVersion")
    mappings("net.fabricmc:yarn:$yarnMappings:v2")
    modImplementation("net.fabricmc:fabric-loader:$loaderVersion")

    // Fabric API. This is technically optional, but you probably want it anyway.
    modImplementation("net.fabricmc.fabric-api:fabric-api:$fabricVersion")

    rust(project(":rust:native-jni"))
}
rustImport {
    baseDir.set("net/randomscientist/soundmod/rust")
    layout.set("hierarchical")
}

tasks {

    val javaVersion = JavaVersion.VERSION_17
    withType<JavaCompile> {
        options.encoding = "UTF-8"
        sourceCompatibility = javaVersion.toString()
        targetCompatibility = javaVersion.toString()
        options.release.set(javaVersion.toString().toInt())
    }

    jar { from("LICENSE") { rename { "${it}_${base.archivesName}" } } }
    java {
        toolchain { languageVersion.set(JavaLanguageVersion.of(javaVersion.toString())) }
        sourceCompatibility = javaVersion
        targetCompatibility = javaVersion
        withSourcesJar()
    }
}
//tasks.getByName("compileJava").dependsOn(tasks.getByPath(":rust:native-jni:build"))
