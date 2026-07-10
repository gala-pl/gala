plugins {
    id("org.jetbrains.intellij") version "1.17.4"
    kotlin("jvm") version "1.9.22"
}

group = "com.gala.lang"
version = "0.1.0"

repositories {
    mavenCentral()
}

intellij {
    pluginName.set("Gala")
    version.set("2024.1")
    type.set("IC")
}

dependencies {
    implementation(kotlin("stdlib"))
}

tasks {
    patchPluginXml {
        sinceBuild.set("241")
        untilBuild.set("241.*")
        changeNotes.set("""
            <h3>v0.1.0</h3>
            <ul>
                <li>Gala language support: syntax highlighting, brace matching, commenter</li>
                <li>File type detection for .gala files</li>
                <li>Default color scheme for quantum/classical token distinction</li>
            </ul>
        """.trimIndent())
    }
}