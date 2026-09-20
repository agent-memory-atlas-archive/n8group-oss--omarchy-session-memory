//! The installer that ships *in* the plugin, and the digest it is anchored to.
//!
//! The Omarchy marketplace's security review of `58cc3f4` said the engine
//! installation chain was not immutably bound to the reviewed snapshot, and
//! it was right. The panel told users to fetch `install.sh` from the `v0.1.0`
//! release and run it. That script carried the binary's SHA-256 — but the
//! script and the digest came from the same mutable release, so a moved tag
//! or a compromised publisher account changes the executable and its claimed
//! digest together. The validated plugin vouched for nothing; the artifact
//! vouched for itself.
//!
//! The fix is to move the anchor into the tree the marketplace actually
//! reads. `omarchy plugin add` does a full `git clone` of this repository
//! into `~/.config/omarchy/plugins/<id>/`, and `omarchy plugin clone`'s
//! `copy_plugin` does `cp -aL "$source_dir/."` when `manifest.json` sits at
//! the repository root — which it does. Either way every root file lands on
//! the user's machine, `install.sh` included. So there is nothing to download
//! before running, and nothing to verify before executing: the script the
//! reviewer read is the script that runs.
//!
//! These tests hold that arrangement in place. They check the script is
//! there, that it carries a digest written *here*, that it enforces it, that
//! it fetches from a tag-pinned URL naming this tree's version, and — when a
//! network is available — that the digest in the tree is the one the release
//! actually published.

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn installer() -> String {
    let p = root().join("install.sh");
    std::fs::read_to_string(&p).unwrap_or_else(|e| {
        panic!(
            "install.sh must ship in the plugin tree, at the repository root \
             where `omarchy plugin add` will copy it onto the user's machine: \
             {} ({e})",
            p.display()
        )
    })
}

/// Script with `#` comment lines removed, so a check cannot be satisfied by
/// a comment describing the thing it is looking for. The shebang goes too —
/// it is a comment as far as any of these assertions are concerned.
fn without_comments(src: &str) -> String {
    src.lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The version this source tree builds, read from `Cargo.toml`.
fn crate_version() -> String {
    std::fs::read_to_string(root().join("Cargo.toml"))
        .expect("Cargo.toml")
        .lines()
        .find_map(|l| {
            l.strip_prefix("version = \"")
                .and_then(|v| v.strip_suffix('"'))
        })
        .expect("a version in Cargo.toml")
        .to_string()
}

/// The value of a `KEY="value"` assignment, outside comments.
fn assignment(src: &str, key: &str) -> Option<String> {
    without_comments(src).lines().find_map(|l| {
        let t = l.trim();
        let rest = t.strip_prefix(key)?.strip_prefix('=')?;
        Some(rest.trim().trim_matches('"').to_string())
    })
}

fn is_sha256(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// The installer is a file in this repository, not a URL in a comment.
///
/// This is the whole point of the change: a user who has the plugin has the
/// installer, because the plugin is a git clone of this tree. If this file
/// ever stops existing the chain silently reverts to "download a script from
/// a mutable release and run it", which is the finding.
#[test]
fn the_installer_ships_at_the_repository_root() {
    let src = installer();
    assert!(
        src.starts_with("#!/bin/sh"),
        "install.sh must be a POSIX sh script — the documented invocation is \
         `sh install.sh`, and Omarchy does not promise bash"
    );
    let bare = without_comments(&src);
    assert!(
        bare.contains("set -eu"),
        "install.sh must `set -eu`: an installer that carries on after a \
         failed download or a failed digest check is worse than none"
    );
    // manifest.json at the root is what makes `copy_plugin` take the whole
    // directory, and what makes this file arrive with it.
    assert!(
        root().join("manifest.json").exists(),
        "manifest.json must stay at the repository root; a manifest moved \
         into a subdirectory changes what Omarchy copies, and install.sh \
         would stop reaching the user's machine"
    );
}

/// The digest is a literal in this file, and this file is the anchor.
///
/// Exactly one, so there is no question which one is enforced, and no way to
/// leave a stale one behind next to a fresh one.
#[test]
fn the_installer_carries_one_binary_digest_written_here() {
    let src = installer();
    let bare = without_comments(&src);

    let digest = assignment(&src, "SHA256").unwrap_or_else(|| {
        panic!("install.sh sets no SHA256=; the binary digest is the anchor and must be a literal here")
    });
    assert!(
        is_sha256(&digest),
        "SHA256 in install.sh is not 64 hex characters: {digest:?}"
    );

    let literals: Vec<&str> = bare
        .split(|c: char| !c.is_ascii_hexdigit())
        .filter(|w| is_sha256(w))
        .collect();
    assert_eq!(
        literals.len(),
        1,
        "install.sh carries {} 64-hex literals; exactly one binary digest \
         must be enforceable, and a second one is a stale anchor waiting to \
         be trusted: {literals:?}",
        literals.len()
    );

    // And it has to be *used*. A digest that is declared and never checked
    // is a comment with quotes around it.
    assert!(
        bare.contains("sha256sum -c"),
        "install.sh declares a digest but never runs `sha256sum -c` against \
         the download; the anchor has to be enforced, not announced"
    );
    assert!(
        !bare.contains(".sha256"),
        "install.sh fetches a checksum file at run time. The published \
         `.sha256` is a convenience for people reading the releases page; \
         verifying the download against a file served by whoever served the \
         download checks for corruption and for nothing else. The digest in \
         this tree is the anchor."
    );
}

/// The binary comes from an exact tag, and that tag is this tree's version.
///
/// `releases/latest/download/…` follows whatever release is newest, so what a
/// user runs and what a reviewer read need not be the same bytes. And the tag
/// has to be *this* version: a plugin whose QML reports 0.2.0 while its
/// installer fetches the v0.1.0 engine is a mismatch the user then has to
/// diagnose.
///
/// This is the assertion that goes red during a release, deliberately. See
/// "Cutting a release" in the README: the digest of a build cannot be known
/// before the build exists, so the version bump and the digest write are two
/// commits, and the tree between them is not a tree to submit for validation.
#[test]
fn the_installer_downloads_from_the_tag_of_this_version() {
    let src = installer();
    let bare = without_comments(&src);
    let version = crate_version();

    let tag = assignment(&src, "TAG")
        .unwrap_or_else(|| panic!("install.sh sets no TAG=; the download must name an exact tag"));
    assert_eq!(
        tag,
        format!("v{version}"),
        "install.sh fetches the {tag} engine, but this tree builds {version}. \
         If a release is in flight, finish it: publish the tag, then write \
         that build's tag and digest into install.sh in one commit, and \
         submit *that* commit for validation."
    );

    assert!(
        !bare.contains("releases/latest/download/"),
        "install.sh points at releases/latest/download/, which follows \
         whatever release is newest; name the tag instead"
    );
    assert!(
        bare.contains("releases/download/${TAG}/")
            || bare.contains(&format!("releases/download/v{version}/")),
        "install.sh must fetch the binary from a tag-pinned release URL:\n{bare}"
    );
    assert!(
        bare.contains("osm-x86_64-unknown-linux-gnu"),
        "install.sh must name the published binary asset"
    );
}

/// `--dry-run` still works, and arguments still reach `osm install`.
///
/// The documented first step is `sh install.sh --dry-run`, which prints every
/// step and touches nothing. That only holds if the script forwards its
/// arguments rather than interpreting them.
#[test]
fn the_installer_passes_its_arguments_through_to_osm_install() {
    let bare = without_comments(&installer());
    assert!(
        bare.contains("install \"$@\""),
        "install.sh must hand its arguments straight to `osm install`, so \
         `sh install.sh --dry-run` and `--prefix` keep working:\n{bare}"
    );
}

/// The digest in the tree is the digest the release published.
///
/// This is the test that makes the anchor real rather than merely present:
/// it fetches `osm-x86_64-unknown-linux-gnu.sha256` for this version's tag
/// and asserts it equals the literal in `install.sh`. If someone writes a
/// digest here that no published binary has, every user who runs the shipped
/// installer gets a refusal — and this test says so first.
///
/// It skips, loudly and successfully, when there is no network or no `curl`.
/// CI has both; a developer on a train does not, and a test that fails for
/// the lack of a network is a test that gets ignored.
#[test]
fn the_in_tree_digest_is_the_one_that_release_published() {
    let version = crate_version();
    let src = installer();
    let want = assignment(&src, "SHA256").expect("SHA256 in install.sh");

    let url = format!(
        "https://github.com/n8group-oss/omarchy-session-memory/releases/download/v{version}/osm-x86_64-unknown-linux-gnu.sha256"
    );

    let out = match std::process::Command::new("curl")
        .args(["-fsSL", "--max-time", "30", &url])
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            eprintln!("skipping: no usable curl on this machine ({e})");
            return;
        }
    };
    if !out.status.success() {
        eprintln!(
            "skipping: could not fetch {url} ({}). Offline, or the v{version} \
             release does not exist yet — which is the expected state between \
             the version bump and the digest write.",
            out.status
        );
        return;
    }

    let body = String::from_utf8_lossy(&out.stdout);
    let published = body
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    assert!(
        is_sha256(&published),
        "{url} did not answer with a SHA-256 line; got:\n{body}"
    );
    assert_eq!(
        published, want,
        "install.sh pins {want} for v{version}, but the release publishes \
         {published}. Either the in-tree digest was never updated after the \
         release, or the release asset has been replaced since — and the \
         second one is the attack this anchor exists to catch. Do not copy \
         the published value over the in-tree one without establishing which \
         it is."
    );
}

/// Nothing in the tree still tells a user to download a script and run it.
///
/// The finding was not only that the released `install.sh` was self-vouching;
/// it was that the plugin *instructed* people onto that path. Shipping a
/// verified installer while the panel still recommends the downloaded one
/// fixes nothing.
#[test]
fn nothing_tells_a_user_to_download_the_installer_and_run_it() {
    for file in ["Menu.qml", "README.md"] {
        let src = std::fs::read_to_string(root().join(file)).expect(file);
        for line in src.lines() {
            let l = line.trim();
            assert!(
                !(l.contains("/install.sh") && l.contains("http")),
                "{file} still points a user at a downloadable install.sh. The \
                 installer ships with the plugin; naming a URL for it puts the \
                 trust anchor back inside the artifact it vouches for:\n{l}"
            );
        }
    }
}

/// And what they point at instead is the copy the plugin actually installs.
///
/// The plugin id is written down exactly once, in `manifest.json`. The README
/// has to show a concrete command, so it names
/// `~/.config/omarchy/plugins/<id>/install.sh` — and that string is checked
/// against the manifest here rather than trusted to have been typed right.
///
/// `Menu.qml` does not restate the id at all. `install.sh` is a sibling of
/// `Menu.qml` by construction, so the panel derives the path from its own URL:
/// that is still correct after `omarchy plugin clone` renames the id, and in a
/// developer's checkout, neither of which a hard-coded path survives.
#[test]
fn the_panel_and_the_readme_name_the_path_the_plugin_installs_to() {
    let raw = std::fs::read_to_string(root().join("manifest.json")).expect("manifest.json");
    let m: serde_json::Value = serde_json::from_str(&raw).expect("manifest is valid JSON");
    let id = m["id"].as_str().expect("an id in manifest.json");
    let want = format!("~/.config/omarchy/plugins/{id}/install.sh");

    let readme = std::fs::read_to_string(root().join("README.md")).expect("README.md");
    assert!(
        readme.contains(&want),
        "README.md does not name the installer where `omarchy plugin add` \
         puts it: {want}"
    );
    assert!(
        root().join("install.sh").exists(),
        "the path the README names has to be a file this repository ships"
    );

    let menu = std::fs::read_to_string(root().join("Menu.qml")).expect("Menu.qml");
    assert!(
        menu.contains("Qt.resolvedUrl(\"install.sh\")"),
        "Menu.qml must derive the installer path from its own URL; the \
         installer is this file's sibling, and a path written out by hand is \
         wrong in every checkout and after every `omarchy plugin clone`"
    );
    for line in menu.lines().filter(|l| !l.trim_start().starts_with("//")) {
        assert!(
            !line.contains("omarchy/plugins/"),
            "Menu.qml hard-codes a plugins path; the id belongs in \
             manifest.json and nowhere else:\n{line}"
        );
    }

    // The copied command is the dry run, and it is shell-safe: the path runs
    // through $HOME, which is not this plugin's to assume is well behaved.
    let at = menu
        .find("function copyInstallCommand(")
        .expect("Menu.qml defines copyInstallCommand");
    let body = &menu[at..];
    let end = body.find("\n  }").expect("copyInstallCommand is closed");
    let body = &body[..end];
    assert!(
        body.contains("shellQuote(root.installerPath)"),
        "the install button builds a command line from an unquoted \
         path:\n{body}"
    );
    assert!(
        body.contains("--dry-run"),
        "the install button should copy the dry run, which is the step that \
         touches nothing:\n{body}"
    );
}
