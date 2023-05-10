plugins {
    id("fr.stardustenterprises.rust.wrapper") version "3.2.5"
}

rust {
    targets += defaultTarget().apply {
        release = true
    }
    cargoInstallTargets.set(true)

}