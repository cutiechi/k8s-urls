use anyhow::{Context, Result};
use clap::Parser;
use k8s_openapi::api::core::v1::{Endpoints, Service};
use kube::{
    api::{Api, ListParams},
    config::Kubeconfig,
    Client, Config,
};
use regex::Regex;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Kubernetes namespace
    #[arg(short, long)]
    namespace: Option<String>,

    /// Path to kubeconfig file
    #[arg(short, long)]
    kubeconfig: Option<PathBuf>,

    /// Filter services by name (regex pattern)
    #[arg(short = 'f', long = "filter")]
    name_filter: Option<String>,
}

fn get_pod_dns(pod_name: &str, service_name: &str, namespace: &str) -> String {
    format!("{pod_name}.{service_name}.{namespace}.svc.cluster.local")
}

fn get_service_dns(service_name: &str, namespace: &str) -> String {
    format!("{service_name}.{namespace}.svc.cluster.local")
}

fn get_protocol_scheme(protocol: &str) -> String {
    match protocol.to_lowercase().as_str() {
        "tcp" => "http".to_string(),
        "udp" => "udp".to_string(),
        other => other.to_lowercase()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // 编译正则表达式（如果提供了）
    let name_regex = if let Some(pattern) = args.name_filter {
        Some(Regex::new(&pattern).context("Invalid regex pattern")?)
    } else {
        None
    };
    
    // 根据参数创建 k8s client
    let client = if let Some(kubeconfig_path) = args.kubeconfig {
        let kubeconfig = Kubeconfig::read_from(&kubeconfig_path)
            .context(format!("Failed to read kubeconfig from {:?}", kubeconfig_path))?;
        let config = Config::from_custom_kubeconfig(kubeconfig, &Default::default())
            .await
            .context("Failed to create config from kubeconfig")?;
        Client::try_from(config).context("Failed to create k8s client")?
    } else {
        Client::try_default().await.context("Failed to create k8s client")?
    };
    
    let namespace = args.namespace.unwrap_or_else(|| String::from("default"));
    
    let services: Api<Service> = Api::namespaced(client.clone(), &namespace);
    let endpoints: Api<Endpoints> = Api::namespaced(client.clone(), &namespace);
    
    println!("\n=== 命名空间: {} ===", namespace);
    if let Some(ref pattern) = name_regex {
        println!("使用名称过滤: {}", pattern.as_str());
    }
    
    let lp = ListParams::default();
    let service_list = services.list(&lp).await?;
    
    for svc in service_list.iter() {
        let svc_name = svc.metadata.name.as_ref().unwrap();
        
        // 如果设置了名称过滤，检查是否匹配
        if let Some(ref regex) = name_regex {
            if !regex.is_match(svc_name) {
                continue;
            }
        }
        
        println!("\n服务: {}", svc_name);
        
        // 打印服务的 DNS 名称
        let svc_dns = get_service_dns(svc_name, &namespace);
        println!("服务 DNS: {}", svc_dns);
        
        if let Some(spec) = &svc.spec {
            let cluster_ip = &spec.cluster_ip;
            
            if let Some(cluster_ip) = cluster_ip {
                if cluster_ip != "None" {
                    println!("类型: ClusterIP Service");
                    if let Some(ports) = &spec.ports {
                        for port in ports {
                            let protocol = port.protocol.as_deref().unwrap_or("TCP");
                            let scheme = get_protocol_scheme(protocol);
                            let port_number = port.port;
                            let port_name = port.name.as_deref().unwrap_or("default");
                            
                            // 打印 ClusterIP URL
                            println!("  ClusterIP URL: {}://{}:{} ({})",
                                scheme,
                                cluster_ip,
                                port_number,
                                port_name
                            );
                            
                            // 打印服务 DNS URL
                            println!("  DNS URL: {}://{}:{} ({})",
                                scheme,
                                svc_dns,
                                port_number,
                                port_name
                            );
                        }
                    }
                } else {
                    println!("类型: Headless Service");
                }
            }
            
            // 获取外部 IP（如果有的话）
            if let Some(status) = &svc.status {
                if let Some(lb) = &status.load_balancer {
                    if let Some(ingress) = &lb.ingress {
                        println!("外部访问点:");
                        for ing in ingress {
                            if let Some(ip) = &ing.ip {
                                if let Some(ports) = &spec.ports {
                                    for port in ports {
                                        let protocol = port.protocol.as_deref().unwrap_or("TCP");
                                        let scheme = get_protocol_scheme(protocol);
                                        println!("  External IP URL: {}://{}:{}", scheme, ip, port.port);
                                    }
                                }
                            }
                            if let Some(hostname) = &ing.hostname {
                                if let Some(ports) = &spec.ports {
                                    for port in ports {
                                        let protocol = port.protocol.as_deref().unwrap_or("TCP");
                                        let scheme = get_protocol_scheme(protocol);
                                        println!("  External Hostname: {}://{}:{}", scheme, hostname, port.port);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // 获取服务对应的端点
        if let Ok(endpoint) = endpoints.get(svc_name).await {
            if let Some(subsets) = endpoint.subsets {
                println!("Pod 端点:");
                for subset in subsets {
                    if let Some(addresses) = subset.addresses {
                        for addr in addresses {
                            let ip = addr.ip;
                            let pod_name = addr
                                .target_ref
                                .as_ref()
                                .and_then(|tr| tr.name.as_ref())
                                .map_or("unknown".to_string(), |s| s.to_string());
                            
                            // 生成 Pod 的 DNS 名称
                            let pod_dns = if let Some(cluster_ip) = &svc.spec.as_ref().and_then(|s| s.cluster_ip.as_ref()) {
                                if cluster_ip.as_str() == "None" {
                                    Some(get_pod_dns(&pod_name, svc_name, &namespace))
                                } else {
                                    None
                                }
                            } else {
                                None
                            };
                            
                            if let Some(ports) = &subset.ports {
                                for port in ports {
                                    let protocol = port.protocol.as_deref().unwrap_or("TCP");
                                    let scheme = get_protocol_scheme(protocol);
                                    println!("  Pod: {}", pod_name);
                                    println!("    IP URL: {}://{}:{}",
                                        scheme,
                                        ip,
                                        port.port
                                    );
                                    
                                    // 对于 Headless Service 的 Pod，打印其 DNS 记录
                                    if let Some(dns) = &pod_dns {
                                        println!("    DNS URL: {}://{}:{}",
                                            scheme,
                                            dns,
                                            port.port
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}
