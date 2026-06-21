mod encryption;
mod paths;
mod policy;
mod safe_id;

#[cfg(test)]
use encryption::NONCE_SIZE;
pub use encryption::{
    decrypt_data, derive_key, encrypt_data, is_encrypted_payload, Aes256GcmEncryptor,
    EncryptionError, EncryptionKey, ENCRYPTED_MAGIC,
};
pub use paths::{
    app_config_dir, app_config_file, app_data_dir, app_data_file, office_app_data_dir,
    office_storage_dir,
};
pub use policy::{
    local_user_content_policy, office_storage_namespace, office_storage_policy, DataBoundary,
    OfficeStorageArea, StorageClass, StorageNamespace, StoragePolicy,
};
pub use safe_id::{
    is_safe_id, sanitize_id, validate_safe_file_name, validate_safe_id, SafeIdError,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn office_storage_policy_is_local_only() {
        let policy = office_storage_policy("tench-docs", OfficeStorageArea::Recovery);

        assert_eq!(policy.namespace.product_id, "tench-docs");
        assert_eq!(policy.namespace.class, StorageClass::UserContent);
        assert_eq!(policy.namespace.name, "recovery");
        assert_eq!(policy.boundary, DataBoundary::LocalOnly);
        assert_eq!(policy.retention_days, Some(7));
    }

    #[test]
    fn office_cache_policy_is_not_user_exportable() {
        let policy = office_storage_policy("tench-slides", OfficeStorageArea::Cache);

        assert_eq!(policy.namespace.class, StorageClass::Cache);
        assert!(!policy.user_exportable);
        assert_eq!(policy.retention_days, Some(30));
    }

    // Edge case tests
    #[test]
    fn all_data_boundary_variants_roundtrip() {
        for variant in [
            DataBoundary::LocalOnly,
            DataBoundary::LocalFirstCloudOptional,
            DataBoundary::CloudRequired,
        ] {
            let serialized = serde::Serialize::serialize(&variant, serde_json::value::Serializer)
                .unwrap()
                .to_string();
            let value: serde_json::Value = serde_json::from_str(&serialized).unwrap();
            let deserialized: DataBoundary = serde::Deserialize::deserialize(value).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn all_storage_class_variants_roundtrip() {
        for variant in [
            StorageClass::Config,
            StorageClass::UserContent,
            StorageClass::Cache,
            StorageClass::Index,
            StorageClass::Secret,
            StorageClass::Telemetry,
        ] {
            let serialized = serde::Serialize::serialize(&variant, serde_json::value::Serializer)
                .unwrap()
                .to_string();
            let value: serde_json::Value = serde_json::from_str(&serialized).unwrap();
            let deserialized: StorageClass = serde::Deserialize::deserialize(value).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn all_office_storage_area_variants_roundtrip() {
        for variant in [
            OfficeStorageArea::Config,
            OfficeStorageArea::RecentFiles,
            OfficeStorageArea::Templates,
            OfficeStorageArea::PromptTemplates,
            OfficeStorageArea::Assets,
            OfficeStorageArea::Cache,
            OfficeStorageArea::Temp,
            OfficeStorageArea::Recovery,
            OfficeStorageArea::Backups,
        ] {
            let serialized = serde::Serialize::serialize(&variant, serde_json::value::Serializer)
                .unwrap()
                .to_string();
            let value: serde_json::Value = serde_json::from_str(&serialized).unwrap();
            let deserialized: OfficeStorageArea = serde::Deserialize::deserialize(value).unwrap();
            assert_eq!(variant, deserialized);
            assert!(!variant.as_str().is_empty());
        }
    }

    #[test]
    fn local_user_content_policy_empty_strings() {
        let policy = local_user_content_policy("", "");
        assert_eq!(policy.namespace.product_id, "");
        assert_eq!(policy.namespace.name, "");
        assert_eq!(policy.namespace.class, StorageClass::UserContent);
        assert_eq!(policy.boundary, DataBoundary::LocalOnly);
        assert!(policy.user_exportable);
        assert_eq!(policy.retention_days, None);
    }

    #[test]
    fn office_storage_namespace_empty_product_id() {
        let ns = office_storage_namespace("", OfficeStorageArea::Temp);
        assert_eq!(ns.product_id, "");
        assert_eq!(ns.class, StorageClass::Cache);
        assert_eq!(ns.name, "temp");
    }

    #[test]
    fn office_storage_policy_all_areas() {
        let areas = [
            OfficeStorageArea::Config,
            OfficeStorageArea::RecentFiles,
            OfficeStorageArea::Templates,
            OfficeStorageArea::PromptTemplates,
            OfficeStorageArea::Assets,
            OfficeStorageArea::Cache,
            OfficeStorageArea::Temp,
            OfficeStorageArea::Recovery,
            OfficeStorageArea::Backups,
        ];
        for area in areas {
            let policy = office_storage_policy("tench-docs", area);
            assert_eq!(policy.namespace.product_id, "tench-docs");
            assert_eq!(policy.boundary, DataBoundary::LocalOnly);
        }
    }

    #[test]
    fn storage_policy_none_retention_days() {
        let policy = office_storage_policy("tench-docs", OfficeStorageArea::Config);
        assert_eq!(policy.retention_days, None);
    }

    #[test]
    fn app_config_dir_with_empty_strings() {
        let path = app_config_dir("", "");
        assert!(!path.as_os_str().is_empty());
    }

    #[test]
    fn app_data_dir_with_empty_strings() {
        let path = app_data_dir("", "");
        assert!(!path.as_os_str().is_empty());
    }

    #[test]
    fn app_config_file_empty_names() {
        let path = app_config_file("", "", "settings.json");
        assert!(path.to_string_lossy().contains("settings.json"));
    }

    #[test]
    fn app_data_file_empty_names() {
        let path = app_data_file("", "", "data.bin");
        assert!(path.to_string_lossy().contains("data.bin"));
    }

    #[test]
    fn office_app_data_dir_empty_product() {
        let path = office_app_data_dir("");
        assert!(path.to_string_lossy().contains("Tench"));
    }

    #[test]
    fn office_storage_dir_empty_product() {
        let path = office_storage_dir("", OfficeStorageArea::Cache);
        assert!(path.to_string_lossy().contains("cache"));
    }

    #[test]
    fn storage_namespace_empty_name() {
        let ns = StorageNamespace {
            product_id: String::new(),
            class: StorageClass::Config,
            name: String::new(),
        };
        let serialized = serde::Serialize::serialize(&ns, serde_json::value::Serializer)
            .unwrap()
            .to_string();
        let value: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        let deserialized: StorageNamespace = serde::Deserialize::deserialize(value).unwrap();
        assert_eq!(ns, deserialized);
    }

    #[test]
    fn storage_policy_none_retention_roundtrip() {
        let policy = StoragePolicy {
            namespace: StorageNamespace {
                product_id: String::new(),
                class: StorageClass::Secret,
                name: String::new(),
            },
            boundary: DataBoundary::CloudRequired,
            encrypted_at_rest: true,
            user_exportable: false,
            retention_days: None,
        };
        let serialized = serde::Serialize::serialize(&policy, serde_json::value::Serializer)
            .unwrap()
            .to_string();
        let value: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        let deserialized: StoragePolicy = serde::Deserialize::deserialize(value).unwrap();
        assert_eq!(policy, deserialized);
    }

    // -----------------------------------------------------------------------
    // Encryption roundtrip tests
    // -----------------------------------------------------------------------

    #[test]
    fn encrypt_decrypt_roundtrip_with_key() {
        let key = derive_key(b"test-key-material");
        let plaintext = b"{\"learner_id\":\"abc123\",\"score\":0.95}";
        let encrypted = encrypt_data(plaintext, &key).expect("encrypt");
        assert!(is_encrypted_payload(&encrypted));
        assert_ne!(&encrypted[ENCRYPTED_MAGIC.len() + NONCE_SIZE..], plaintext);
        let decrypted = decrypt_data(&encrypted, &key).expect("decrypt");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_decrypt_empty_data() {
        let key = derive_key(b"key-for-empty");
        let encrypted = encrypt_data(&[], &key).expect("encrypt");
        assert!(is_encrypted_payload(&encrypted));
        let decrypted = decrypt_data(&encrypted, &key).expect("decrypt");
        assert!(decrypted.is_empty());
    }

    #[test]
    fn encrypt_decrypt_large_payload() {
        let key = derive_key(b"large-payload-key");
        let plaintext = vec![0xAB_u8; 10_000];
        let encrypted = encrypt_data(&plaintext, &key).expect("encrypt");
        let decrypted = decrypt_data(&encrypted, &key).expect("decrypt");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn different_nonces_produce_different_ciphertexts() {
        let key = derive_key(b"nonce-test-key");
        let plaintext = b"same data";
        let encrypted_a = encrypt_data(plaintext, &key).expect("encrypt a");
        let encrypted_b = encrypt_data(plaintext, &key).expect("encrypt b");
        // Different nonces mean the ciphertexts should differ.
        assert_ne!(encrypted_a, encrypted_b);
        // But both decrypt to the same plaintext.
        assert_eq!(
            decrypt_data(&encrypted_a, &key).expect("decrypt a"),
            decrypt_data(&encrypted_b, &key).expect("decrypt b")
        );
    }

    #[test]
    fn wrong_key_fails_to_decrypt() {
        let key_a = derive_key(b"correct-key");
        let key_b = derive_key(b"wrong-key");
        let plaintext = b"secret data";
        let encrypted = encrypt_data(plaintext, &key_a).expect("encrypt");
        let result = decrypt_data(&encrypted, &key_b);
        // AES-256-GCM authentication should fail with the wrong key.
        assert_eq!(result.unwrap_err(), EncryptionError::DecryptionFailed);
    }

    #[test]
    fn decrypt_too_short_returns_error() {
        let key = derive_key(b"any-key");
        let result = decrypt_data(&[0u8; 4], &key);
        assert_eq!(result.unwrap_err(), EncryptionError::DataTooShort);
    }

    #[test]
    fn decrypt_invalid_magic_returns_error() {
        let key = derive_key(b"any-key");
        let mut data = vec![0u8; ENCRYPTED_MAGIC.len() + NONCE_SIZE + 10];
        // Fill with non-magic bytes
        data[..ENCRYPTED_MAGIC.len()].copy_from_slice(b"BADMAGIC");
        let result = decrypt_data(&data, &key);
        assert_eq!(result.unwrap_err(), EncryptionError::InvalidMagic);
    }

    #[test]
    fn encryption_key_from_machine_is_deterministic() {
        let key_a = EncryptionKey::from_machine();
        let key_b = EncryptionKey::from_machine();
        assert_eq!(key_a.derived_key(), key_b.derived_key());
    }

    #[test]
    fn encryption_key_roundtrip() {
        let enc_key = EncryptionKey::from_machine();
        let derived = enc_key.derived_key();
        let plaintext = b"learner progress data";
        let encrypted = encrypt_data(plaintext, &derived).expect("encrypt");
        let decrypted = decrypt_data(&encrypted, &derived).expect("decrypt");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn is_encrypted_payload_detects_magic() {
        let key = derive_key(b"detect-key");
        let encrypted = encrypt_data(b"test", &key).expect("encrypt");
        assert!(is_encrypted_payload(&encrypted));
        assert!(!is_encrypted_payload(b"{\"json\":true}"));
        assert!(!is_encrypted_payload(&[]));
    }

    // -----------------------------------------------------------------------
    // SafeId validation tests
    // -----------------------------------------------------------------------

    #[test]
    fn is_safe_id_rejects_empty_string() {
        assert!(!is_safe_id(""));
    }

    #[test]
    fn is_safe_id_accepts_alphanumeric() {
        assert!(is_safe_id("abc123"));
    }

    #[test]
    fn is_safe_id_accepts_hyphens() {
        assert!(is_safe_id("my-id"));
    }

    #[test]
    fn is_safe_id_accepts_underscores() {
        assert!(is_safe_id("my_id"));
    }

    #[test]
    fn is_safe_id_accepts_mixed_safe_characters() {
        assert!(is_safe_id("My-Id_123"));
    }

    #[test]
    fn is_safe_id_rejects_path_traversal_dot_dot() {
        assert!(!is_safe_id(".."));
    }

    #[test]
    fn is_safe_id_rejects_forward_slash() {
        assert!(!is_safe_id("a/b"));
    }

    #[test]
    fn is_safe_id_rejects_backslash() {
        assert!(!is_safe_id("a\\b"));
    }

    #[test]
    fn is_safe_id_rejects_space() {
        assert!(!is_safe_id("a b"));
    }

    #[test]
    fn is_safe_id_rejects_unicode() {
        assert!(!is_safe_id("naïve"));
    }

    #[test]
    fn is_safe_id_rejects_dot() {
        assert!(!is_safe_id("a.b"));
    }

    #[test]
    fn is_safe_id_rejects_special_characters() {
        assert!(!is_safe_id("a!b@c#"));
    }

    #[test]
    fn validate_safe_id_rejects_empty() {
        let err = validate_safe_id("").unwrap_err();
        assert_eq!(err.id, "");
        assert!(err.reason.contains("empty"), "reason: {}", err.reason);
    }

    #[test]
    fn validate_safe_id_accepts_valid() {
        assert!(validate_safe_id("valid-id_123").is_ok());
    }

    #[test]
    fn validate_safe_id_rejects_path_traversal() {
        let err = validate_safe_id("../etc/passwd").unwrap_err();
        assert!(err.reason.contains("disallowed"), "reason: {}", err.reason);
    }

    #[test]
    fn validate_safe_id_rejects_slash() {
        assert!(validate_safe_id("a/b").is_err());
    }

    #[test]
    fn validate_safe_file_name_rejects_path_escape_security_regression() {
        for file_name in ["../profile.json", "a/b.json", "a\\b.json", "..", "CON.json"] {
            assert!(
                validate_safe_file_name(file_name).is_err(),
                "{file_name} should be rejected"
            );
        }
        assert!(validate_safe_file_name("study_state.json").is_ok());
        assert!(validate_safe_file_name("backup-1.manifest.json").is_ok());
    }

    #[test]
    fn validate_safe_id_rejects_backslash() {
        assert!(validate_safe_id("a\\b").is_err());
    }

    #[test]
    fn validate_safe_id_rejects_space() {
        assert!(validate_safe_id("a b").is_err());
    }

    #[test]
    fn validate_safe_id_rejects_unicode() {
        assert!(validate_safe_id("日本語").is_err());
    }

    #[test]
    fn sanitize_id_strips_special_characters() {
        assert_eq!(sanitize_id("hello world!", "fallback"), "helloworld");
    }

    #[test]
    fn sanitize_id_preserves_safe_characters() {
        assert_eq!(sanitize_id("my-id_123", "fallback"), "my-id_123");
    }

    #[test]
    fn sanitize_id_returns_fallback_when_empty_input() {
        assert_eq!(sanitize_id("", "fallback"), "fallback");
    }

    #[test]
    fn sanitize_id_returns_fallback_when_all_stripped() {
        assert_eq!(sanitize_id("!@#$%", "fallback"), "fallback");
    }

    #[test]
    fn sanitize_id_strips_path_traversal() {
        // Dots are stripped, slashes are stripped, only safe chars remain.
        assert_eq!(sanitize_id("../etc/passwd", "fb"), "etcpasswd");
    }

    #[test]
    fn sanitize_id_strips_unicode() {
        assert_eq!(sanitize_id("naïve", "fb"), "nave");
    }

    #[test]
    fn sanitize_id_strips_spaces() {
        assert_eq!(sanitize_id("a b c", "fb"), "abc");
    }

    #[test]
    fn sanitize_id_strips_backslashes() {
        assert_eq!(sanitize_id("a\\b\\c", "fb"), "abc");
    }

    // -----------------------------------------------------------------------
    // Security regression tests
    // -----------------------------------------------------------------------

    #[test]
    fn safe_id_rejects_path_traversal_security() {
        assert!(!is_safe_id("../etc/passwd"));
        assert!(!is_safe_id("foo/../../../bar"));
        assert!(!is_safe_id("..\\windows\\system32"));
        assert!(validate_safe_id("../etc/passwd").is_err());
        assert!(validate_safe_id("..").is_err());
    }

    #[test]
    fn sanitize_id_fallback_security() {
        // All characters are disallowed → fallback is used.
        assert_eq!(sanitize_id("!@#$%", "fallback"), "fallback");
        // Mix of allowed and disallowed → only allowed chars kept.
        assert_eq!(sanitize_id("a!b@c#", "fallback"), "abc");
        // Empty input → fallback.
        assert_eq!(sanitize_id("", "fallback"), "fallback");
    }

    // -----------------------------------------------------------------------
    // Release validation tests
    // -----------------------------------------------------------------------

    #[test]
    fn encrypted_data_roundtrip_release() {
        let key = derive_key(b"release-validation-key");
        let plaintext = b"{\"release\":\"1.0.0\",\"channel\":\"stable\"}";
        let encrypted = encrypt_data(plaintext, &key).expect("encrypt");
        assert!(is_encrypted_payload(&encrypted));
        let decrypted = decrypt_data(&encrypted, &key).expect("decrypt");
        assert_eq!(decrypted, plaintext);
    }
}
