plugins {
    id("fr.stardustenterprises.rust.wrapper") version "3.2.5"
}

rust {
    release.set(true)
    targets += defaultTarget()
    cargoInstallTargets.set(true)

}