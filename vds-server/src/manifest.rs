#[derive(Debug, PartialEq, Eq)]
pub struct Version(u32, u32, u32);

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct Video {
    name: String,
    id: uuid::Uuid,
    #[serde(deserialize_with = "deserialize_uri")]
    #[serde(serialize_with = "serialize_uri")]
    uri: http::Uri,
    sha256: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct Section {
    name: String,
    content: Vec<Video>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct ManifestFile {
    name: String,
    date: chrono::NaiveDate,
    version: Version,
    sections: Vec<Section>,
}

fn serialize_uri<S>(uri: &http::Uri, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let s = uri.to_string();
    serializer.serialize_str(&s)
}

fn deserialize_uri<'de, D>(deserializer: D) -> Result<http::Uri, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_str(uri::Visitor {})
}

impl<'de> serde::Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(version::Visitor {})
    }
}

impl serde::Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("v{}.{}.{}", self.0, self.1, self.2))
    }
}

mod uri {
    pub struct Visitor {}

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = http::Uri;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("A URI")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            v.parse().map_err(E::custom)
        }
    }
}

mod version {
    pub struct Visitor {}

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = super::Version;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("A version number of the form vX.Y.Z")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let Some(("", right)) = v.split_once("v") else {
                return Err(E::custom("Version does not start with \"v\""));
            };

            let components: Result<Vec<u32>, _> = right.split(".").map(|c| c.parse()).collect();
            let Ok(components) = components else {
                return Err(E::custom("Invalid version string."));
            };

            if components.len() != 3 {
                return Err(E::custom("Invalid version string."));
            }

            Ok(super::Version(components[0], components[1], components[2]))
        }
    }
}

#[cfg(test)]
pub mod test {
    use std::str::FromStr;

    use super::*;
    use googletest::prelude::*;

    #[googletest::gtest]
    fn deserialize_version() -> googletest::Result<()> {
        let version = serde_json::from_str::<Version>(r#""v1.2.3""#).or_fail()?;
        expect_that!(version, eq(&Version(1, 2, 3)));
        let version = serde_json::from_str::<Version>(r#""v432.224.8234""#).or_fail()?;
        expect_that!(version, eq(&Version(432, 224, 8234)));

        Ok(())
    }

    #[googletest::gtest]
    fn deserialize_version_incorrect_format() -> googletest::Result<()> {
        let testcases = [
            r#""1.2.3""#,
            r#""v.2.3""#,
            r#""va1.2.3""#,
            r#""v1..3""#,
            r#""v1.3.3.4""#,
            r#""v1.3.""#,
            r#""v1.3""#,
            r#""a1.3a""#,
            r#""v1.3.3.a""#,
            r#""v1.3.3a""#,
        ];

        for testcase in testcases {
            expect_that!(serde_json::from_str::<Version>(testcase), err(anything()));
        }

        Ok(())
    }

    #[googletest::gtest]
    fn serialize_version() -> googletest::Result<()> {
        let expected = r#""v1.2.3""#;
        let version = serde_json::to_string(&Version(1, 2, 3)).or_fail()?;
        expect_that!(version, eq(expected));
        Ok(())
    }

    #[googletest::gtest]
    fn deserialize_video() -> googletest::Result<()> {
        let serialized = r#"{
            "name": "Linear equations",
            "id": "bf978778-1c5d-44b3-b2c1-1cc253563799",
            "uri": "s3://bucket/linear-equations.mp4",
            "sha256": "0b88b2dec2be5e2ef74022ef6a8023232e28374d67e917b76f9bb607e691f327"
        }"#;

        let video: Video = serde_json::from_str(serialized).unwrap();
        expect_that!(
            video,
            eq(&Video {
                name: "Linear equations".to_string(),
                id: uuid::Uuid::from_str("bf978778-1c5d-44b3-b2c1-1cc253563799").or_fail()?,
                uri: "s3://bucket/linear-equations.mp4".parse().or_fail()?,
                sha256: "0b88b2dec2be5e2ef74022ef6a8023232e28374d67e917b76f9bb607e691f327"
                    .to_string(),
            })
        );
        Ok(())
    }

    #[googletest::gtest]
    fn deserialize_section() -> googletest::Result<()> {
        let serialized = r#"{
            "name": "Equations",
            "content": [
                {
                    "name": "Linear equations",
                    "id": "bf978778-1c5d-44b3-b2c1-1cc253563799",
                    "uri": "s3://bucket/linear-equations.mp4",
                    "sha256": "0b88b2dec2be5e2ef74022ef6a8023232e28374d67e917b76f9bb607e691f327"
                },
                {
                    "name": "Quadratic equations",
                    "id": "5eb9e089-79cf-478d-9121-9ca3e7bb1d4a",
                    "uri": "s3://bucket/quadratic-equations.mp4",
                    "sha256": "8f9e3a4ae7d86c4abdf731a947fc90b607b82a0362da0b312e3b644defedb81f"
                },
                {
                    "name": "Cubic equations",
                    "id": "9e0f44b6-3dc6-4f56-8c9f-7e28feac1d03",
                    "uri": "s3://bucket/cubic-equations.mp4",
                    "sha256": "8b9522ce42fb02dd100b575714d935a4502872afccee80f7a65d466389a5bef8"
                }
            ]
        }"#;

        let section: Section = serde_json::from_str(serialized).unwrap();
        expect_that!(
            section,
            eq(&Section {
                name: "Equations".to_string(),
                content: vec![
                    Video {
                        name: "Linear equations".to_string(),
                        id: uuid::Uuid::from_str("bf978778-1c5d-44b3-b2c1-1cc253563799")
                            .or_fail()?,
                        uri: "s3://bucket/linear-equations.mp4".parse().or_fail()?,
                        sha256: "0b88b2dec2be5e2ef74022ef6a8023232e28374d67e917b76f9bb607e691f327"
                            .to_string(),
                    },
                    Video {
                        name: "Quadratic equations".to_string(),
                        id: uuid::Uuid::from_str("5eb9e089-79cf-478d-9121-9ca3e7bb1d4a")
                            .or_fail()?,
                        uri: "s3://bucket/quadratic-equations.mp4".parse().or_fail()?,
                        sha256: "8f9e3a4ae7d86c4abdf731a947fc90b607b82a0362da0b312e3b644defedb81f"
                            .to_string(),
                    },
                    Video {
                        name: "Cubic equations".to_string(),
                        id: uuid::Uuid::from_str("9e0f44b6-3dc6-4f56-8c9f-7e28feac1d03")
                            .or_fail()?,
                        uri: "s3://bucket/cubic-equations.mp4".parse().or_fail()?,
                        sha256: "8b9522ce42fb02dd100b575714d935a4502872afccee80f7a65d466389a5bef8"
                            .to_string(),
                    },
                ]
            })
        );
        Ok(())
    }

    #[googletest::gtest]
    fn deserialize_manifest() -> googletest::Result<()> {
        let serialized = r#"{
    "name": "High school video distribution list",
    "date": "2025-10-10",
    "version": "v1.0.0",
    "sections": [
        {
            "name": "Equations",
            "content": [
                {
                    "name": "Linear equations",
                    "id": "bf978778-1c5d-44b3-b2c1-1cc253563799",
                    "uri": "s3://bucket/linear-equations.mp4",
                    "sha256": "0b88b2dec2be5e2ef74022ef6a8023232e28374d67e917b76f9bb607e691f327"
                },
                {
                    "name": "Quadratic equations",
                    "id": "5eb9e089-79cf-478d-9121-9ca3e7bb1d4a",
                    "uri": "s3://bucket/quadratic-equations.mp4",
                    "sha256": "8f9e3a4ae7d86c4abdf731a947fc90b607b82a0362da0b312e3b644defedb81f"
                },
                {
                    "name": "Cubic equations",
                    "id": "9e0f44b6-3dc6-4f56-8c9f-7e28feac1d03",
                    "uri": "s3://bucket/cubic-equations.mp4",
                    "sha256": "8b9522ce42fb02dd100b575714d935a4502872afccee80f7a65d466389a5bef8"
                }
            ]
        },
        {
            "name": "Integration",
            "content": [
                {
                    "name": "Riemann sum",
                    "id": "eddb4450-a9ff-4a4b-ad81-2a8b78998405",
                    "uri": "s3://bucket/riemann-sum.mp4",
                    "sha256": "a6d3b80cd14f78b21ffbf5995bbda38ad8834459557782d245ed720134d36fc4"
                },
                {
                    "name": "List of integrals",
                    "id": "f47e6cdc-1bcf-439a-9ea4-038dc7153648",
                    "uri": "s3://bucket/list-of-integrals.mp4",
                    "sha256": "98780990e94fb55d0b88ebcd78fe82f069eac547731a4b0822332d826c970aec"
                }
            ]
        }
    ]
}"#;

        let manifest: ManifestFile = serde_json::from_str(serialized).or_fail()?;
        expect_that!(
            manifest,
            eq(&ManifestFile {
                name: "High school video distribution list".to_string(),
                date: chrono::NaiveDate::from_str("2025-10-10").or_fail()?,
                version: Version(1, 0, 0),
                sections: vec![
                    Section {
                        name: "Equations".to_string(),
                        content: vec![
                        Video {
                            name: "Linear equations".to_string(),
                            id: uuid::Uuid::from_str("bf978778-1c5d-44b3-b2c1-1cc253563799")
                                .or_fail()?,
                            uri: "s3://bucket/linear-equations.mp4".parse().or_fail()?,
                            sha256:
                                "0b88b2dec2be5e2ef74022ef6a8023232e28374d67e917b76f9bb607e691f327"
                                    .to_string(),
                        },
                        Video {
                            name: "Quadratic equations".to_string(),
                            id: uuid::Uuid::from_str("5eb9e089-79cf-478d-9121-9ca3e7bb1d4a")
                                .or_fail()?,
                            uri: "s3://bucket/quadratic-equations.mp4".parse().or_fail()?,
                            sha256:
                                "8f9e3a4ae7d86c4abdf731a947fc90b607b82a0362da0b312e3b644defedb81f"
                                    .to_string(),
                        },
                        Video {
                            name: "Cubic equations".to_string(),
                            id: uuid::Uuid::from_str("9e0f44b6-3dc6-4f56-8c9f-7e28feac1d03")
                                .or_fail()?,
                            uri: "s3://bucket/cubic-equations.mp4".parse().or_fail()?,
                            sha256:
                                "8b9522ce42fb02dd100b575714d935a4502872afccee80f7a65d466389a5bef8"
                                    .to_string(),
                        },
                    ]
                    },
                    Section {
                        name: "Integration".to_string(),
                        content: vec![
                        Video {
                            name: "Riemann sum".to_string(),
                            id: uuid::Uuid::from_str("eddb4450-a9ff-4a4b-ad81-2a8b78998405")
                                .or_fail()?,
                            uri: "s3://bucket/riemann-sum.mp4".parse().or_fail()?,
                            sha256:
                                "a6d3b80cd14f78b21ffbf5995bbda38ad8834459557782d245ed720134d36fc4"
                                    .to_string(),
                        },
                        Video {
                            name: "List of integrals".to_string(),
                            id: uuid::Uuid::from_str("f47e6cdc-1bcf-439a-9ea4-038dc7153648")
                                .or_fail()?,
                            uri: "s3://bucket/list-of-integrals.mp4".parse().or_fail()?,
                            sha256:
                                "98780990e94fb55d0b88ebcd78fe82f069eac547731a4b0822332d826c970aec"
                                    .to_string(),
                        },
                    ]
                    }
                ],
            })
        );
        Ok(())
    }
}
