plugins {
    kotlin("jvm") version "2.1.0"
    `maven-publish`
    signing
}

group = "rs.in.ininids"
version = "2.0.1" // x-release-please-version

repositories {
    mavenCentral()
}

dependencies {

}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(17))
    }
    withSourcesJar()
    withJavadocJar()
}

tasks.test {
    useJUnitPlatform()
}

publishing {
    publications {
        create<MavenPublication>("mavenJava") {
            from(components["java"])

            pom {
                name.set("AAM-JV")
                description.set("JNI bindings for aam-rs configuration parser")
                url.set("https://github.com/ininids/aam-rs")

                licenses {
                    license {
                        name.set("MIT License")
                        url.set("https://opensource.org/licenses/MIT")
                    }
                    license {
                        name.set("APACHE-2.0")
                        url.set("http://www.apache.org/licenses/LICENSE-2.0")
                    }
                }
                developers {
                    developer {
                        id.set("ininids")
                        name.set("Nikita Goncharov")
                        email.set("ininids@ininids.in.rs")
                    }
                }
                scm {
                    connection.set("scm:git:git://github.com/ininids/aam-rs.git")
                    developerConnection.set("scm:git:ssh://github.com:ininids/aam-rs.git")
                    url.set("https://github.com/ininids/aam-rs/tree/main")
                }
            }
        }
    }

    repositories {
        maven {
            name = "Staging"
            url = uri(layout.buildDirectory.dir("staging-repo"))
        }
        maven {
            name = "GitHubPackages"
            url = uri("https://maven.pkg.github.com/ininids/aam-rs")
            credentials {
                username = System.getenv("GITHUB_ACTOR")
                password = System.getenv("GITHUB_TOKEN")
            }
        }
    }
}

val signingKey = findProperty("signingInMemoryKey") as String?
val signingPassword = findProperty("signingInMemoryKeyPassword") as String?

signing {
    isRequired = signingKey != null
    if (signingKey != null) {
        useInMemoryPgpKeys(signingKey, signingPassword)
    }
    sign(publishing.publications["mavenJava"])
}