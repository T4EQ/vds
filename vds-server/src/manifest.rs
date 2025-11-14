#[derive(Debug, PartialEq, Eq)]
pub struct Version(u32, u32, u32);

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct Video {
    name: String,
    id: uuid::Uuid,
    uri: String, // TODO: should be URI
    sha256: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct Section {
    name: String,
    content: Vec<Video>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ManifestFile {
    name: String,
    date: chrono::NaiveDateTime,

    version: Version,
    sections: Vec<Section>,
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
            r#""v1.3.3""#,
        ];

        for testcase in testcases {
            expect_that!(serde_json::from_str::<Version>(testcase), err(anything()));
        }

        Ok(())
    }

    #[googletest::gtest]
    fn serialize_version() {
        let expected = r#""v1.2.3""#;
        let version = serde_json::to_string(&Version(1, 2, 3)).unwrap();
        assert_eq!(version, expected);
    }

    #[googletest::gtest]
    fn deserialize_video() {
        let serialized = r#"{
            "name": "Linear equations",
            "id": "bf978778-1c5d-44b3-b2c1-1cc253563799",
            "uri": "s3://bucket/linear-equations.mp4",
            "sha256": "0b88b2dec2be5e2ef74022ef6a8023232e28374d67e917b76f9bb607e691f327"
        }"#;
        let video: Video = serde_json::from_str(serialized).unwrap();
        assert_eq!(video.name, "Linear equations");
        assert_eq!(
            video.id,
            uuid::Uuid::from_str("bf978778-1c5d-44b3-b2c1-1cc253563799").unwrap()
        );
        assert_eq!(video.uri, "s3://bucket/linear-equations.mp4");
        assert_eq!(
            video.sha256,
            "0b88b2dec2be5e2ef74022ef6a8023232e28374d67e917b76f9bb607e691f327"
        );
    }
}
