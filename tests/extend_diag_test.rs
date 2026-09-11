#[test]
fn diag_extended_from_same_file() {
    let tmp = std::env::temp_dir().join(format!("ext-same-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).expect("create tmp");

    std::fs::write(tmp.join("input.scss"), "@use \"other\";\n\nin-input {@extend in-other-extender}\n").expect("write");
    std::fs::write(tmp.join("_other.scss"), "in-other-extender {@extend in-other-extendee}\n\nin-other-extendee {x: y}\n").expect("write");

    let input = tmp.join("input.scss");
    let result = sasspile::compile_file_with_load_paths(
        &input,
        sasspile::OutputStyle::Expanded,
        vec![tmp.clone()],
    );
    match &result {
        Ok(css) => { std::fs::write("/tmp/ext_same_ok.css", css).ok(); }
        Err(e) => { std::fs::write("/tmp/ext_same_err.txt", format!("{e}")).ok(); }
    }
    let _ = result;
}

#[test]
fn diag_extended_from_other_file() {
    let tmp = std::env::temp_dir().join(format!("ext-other-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).expect("create tmp");

    std::fs::write(tmp.join("input.scss"), "@use \"midstream\";\n\nin-input {@extend in-midstream}\n").expect("write");
    std::fs::write(tmp.join("_midstream.scss"), "@use \"upstream\";\n\nin-midstream {@extend in-upstream}\n").expect("write");
    std::fs::write(tmp.join("_upstream.scss"), "in-upstream {x: y}\n").expect("write");

    let input = tmp.join("input.scss");
    let result = sasspile::compile_file_with_load_paths(
        &input,
        sasspile::OutputStyle::Expanded,
        vec![tmp.clone()],
    );
    match &result {
        Ok(css) => { std::fs::write("/tmp/ext_other_ok.css", css).ok(); }
        Err(e) => { std::fs::write("/tmp/ext_other_err.txt", format!("{e}")).ok(); }
    }
    let _ = result;
}

#[test]
fn diag_optional_and_mandatory_same_file() {
    let tmp = std::env::temp_dir().join(format!("opt-mand-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).expect("create tmp");

    std::fs::write(tmp.join("input.scss"), "@use \"other\";\n\nin-input {\n  @extend %-in-other !optional;\n  @extend %-in-other;\n}\n").expect("write");
    std::fs::write(tmp.join("_other.scss"), "%-in-other {x: y}\n\nin-other {@extend %-in-other}\n").expect("write");

    let input = tmp.join("input.scss");
    let result = sasspile::compile_file_with_load_paths(
        &input,
        sasspile::OutputStyle::Expanded,
        vec![tmp.clone()],
    );
    match &result {
        Ok(css) => { std::fs::write("/tmp/opt_mand_ok.css", css).ok(); }
        Err(e) => { std::fs::write("/tmp/opt_mand_err.txt", format!("{e}")).ok(); }
    }
    let _ = result;
}

#[test]
fn diag_extend_into_pseudo() {
    // Case from sass-spec/pseudo.hrx:
    // :is(midstream) {@extend upstream}
    // downstream {@extend midstream}
    // upstream {a: b}
    //
    // Expected:
    // upstream, :is(midstream), :is(midstream, downstream) {
    //   a: b;
    // }
    let input = ":is(midstream) {@extend upstream}\n\ndownstream {@extend midstream}\n\nupstream {a: b}\n";
    let result = sasspile::compile(input, sasspile::OutputStyle::Expanded);
    match &result {
        Ok(css) => { std::fs::write("/tmp/ext_pseudo_ok.css", css).ok(); }
        Err(e) => { std::fs::write("/tmp/ext_pseudo_err.txt", format!("{e}")).ok(); }
    }
    let _ = result;
}

#[test]
fn diag_extend_within_pseudo_module_two_files() {
    // Case from sass-spec/midstream_extend_within_pseudoselector.hrx (two_files/is)
    // input.scss:
    //   @use "upstream";
    //   :is(in-midstream) {@extend in-upstream}
    //   in-input { @extend in-midstream; y: z; }
    // _upstream.scss:
    //   in-upstream {a: b}
    //
    // Expected:
    // in-upstream, :is(in-midstream, in-input) {
    //   a: b;
    // }
    // in-input {
    //   y: z;
    // }
    let tmp = std::env::temp_dir().join(format!("pseudo-mod-2f-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).expect("create tmp");

    std::fs::write(
        tmp.join("input.scss"),
        "@use \"upstream\";\n\n:is(in-midstream) {@extend in-upstream}\n\nin-input {\n  @extend in-midstream;\n  y: z;\n}\n",
    )
    .expect("write input");
    std::fs::write(tmp.join("_upstream.scss"), "in-upstream {a: b}\n").expect("write upstream");

    let input = tmp.join("input.scss");
    let result = sasspile::compile_file_with_load_paths(
        &input,
        sasspile::OutputStyle::Expanded,
        vec![tmp.clone()],
    );
    match &result {
        Ok(css) => {
            std::fs::write("/tmp/pseudo_mod_2f_ok.css", css).ok();
            tracing::info!("\n=== TWO FILES OUTPUT ===\n{css}");
        }
        Err(e) => {
            std::fs::write("/tmp/pseudo_mod_2f_err.txt", format!("{e}")).ok();
            tracing::warn!("ERROR: {e}");
        }
    }
    let _ = result;
}

#[test]
fn diag_extend_within_pseudo_module_three_files() {
    // Case from sass-spec/midstream_extend_within_pseudoselector.hrx (three_files/is)
    // input.scss:
    //   @use "midstream";
    //   in-input { @extend in-midstream; y: z; }
    // _midstream.scss:
    //   @use "upstream";
    //   :is(in-midstream) {@extend in-upstream}
    // _upstream.scss:
    //   in-upstream {a: b}
    //
    // Expected:
    // in-upstream, :is(in-midstream, in-input) {
    //   a: b;
    // }
    // in-input {
    //   y: z;
    // }
    let tmp = std::env::temp_dir().join(format!("pseudo-mod-3f-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).expect("create tmp");

    std::fs::write(
        tmp.join("input.scss"),
        "@use \"midstream\";\n\nin-input {\n  @extend in-midstream;\n  y: z;\n}\n",
    )
    .expect("write input");
    std::fs::write(
        tmp.join("_midstream.scss"),
        "@use \"upstream\";\n\n:is(in-midstream) {@extend in-upstream}\n",
    )
    .expect("write midstream");
    std::fs::write(tmp.join("_upstream.scss"), "in-upstream {a: b}\n").expect("write upstream");

    let input = tmp.join("input.scss");
    let result = sasspile::compile_file_with_load_paths(
        &input,
        sasspile::OutputStyle::Expanded,
        vec![tmp.clone()],
    );
    match &result {
        Ok(css) => {
            std::fs::write("/tmp/pseudo_mod_3f_ok.css", css).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/pseudo_mod_3f_err.txt", format!("{e}")).ok();
        }
    }
    let _ = result;
}
