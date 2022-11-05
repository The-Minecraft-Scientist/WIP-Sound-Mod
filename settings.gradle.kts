pluginManagement.repositories {
    mavenLocal()
    gradlePluginPortal()
    maven {
        name = "FabricMC"
        url = uri("https://maven.fabricmc.net/")
    }
}

rootProject.name = "wip-sound-mod"
include("soundmod-natives","soundmod")
