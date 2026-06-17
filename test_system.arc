// 声明式配置：描述系统的预期状态
system {
    package "nginx" {
        version = "1.24";
        state = "enabled";
    }
    
    service "arcaea-core" {
        bind = "0.0.0.0:8080";
        auto_restart = true;
    }
}

print("System configuration loaded successfully.");