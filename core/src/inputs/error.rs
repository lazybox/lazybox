error_chain! {
    types {}
    links {}

    foreign_links {
        Io(::std::io::Error);
        Yaml(::yaml_rust::ScanError);
    }

    errors {
        RulesFormat
        InterfaceFormat
        ConditionFormat
        UnknownInterface
    }
}