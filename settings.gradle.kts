pluginManagement.repositories {
    mavenLocal()
    gradlePluginPortal()
    maven {
        name = "FabricMC"
        url = uri("https://maven.fabricmc.net/")
    }
}

rootProject.name = "sound-mod-project"
include("soundmod-natives","soundmod")
