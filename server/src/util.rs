use std::env;

pub fn default_dir() -> String {
    String::from(env!("HOME")) + "/.slackrypt-server"
}

pub fn get_env_var(var: &str, default: &str) -> String {
    match env::var(var) {
        Ok(value) => value,
        Err(_e) => String::from(default),
    }
}

pub fn get_init_sh_cmd(base_url: &str) -> String {
    let mut cmd = String::from("#!/bin/sh\n");
    cmd.push_str("echo \"server_base_url=");
    cmd.push_str(base_url);
    cmd.push_str("\" >> ~/.slackrypt/slackrypt.properties");
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_dir() {
        let s: String = default_dir();
        assert_eq!(true, s.ends_with(".slackrypt-server"));
    }

    #[test]
    fn test_get_init_sh_cmd() {
        let expected: &str = "#!/bin/sh\necho \"server_base_url=http://127.0.0.1:8080\" >> ~/.slackrypt/slackrypt.properties";
        assert_eq!(expected, get_init_sh_cmd("http://127.0.0.1:8080"))
    }
}
