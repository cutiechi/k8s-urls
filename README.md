# k8s-urls

A command-line tool written in Rust for discovering and displaying Kubernetes service and pod network endpoints across different namespaces.

## Features

- 🔍 List Kubernetes service URLs across different namespaces
- 🌐 Support for both ClusterIP and Headless services
- 📝 Display service and pod DNS records
- 🔧 Flexible kubeconfig file selection
- 🎯 Filter services by name using regex patterns
- 🔒 Uses standard Kubernetes authentication

## Installation

### Download Pre-built Binary

You can download the pre-built binary for your platform from the [GitHub Releases](https://github.com/cutiechi/k8s-urls/releases) page. We provide binaries for:

- Linux (x86_64, ARM64)
- macOS (Intel, Apple Silicon)

After downloading, extract the archive and move the binary to a directory in your PATH:

```bash
# Example for macOS ARM64
tar xzf k8s-urls-aarch64-apple-darwin.tar.gz
sudo mv k8s-urls /usr/local/bin/
```

### From Source

Ensure you have Rust installed ([rustup](https://rustup.rs/)), then:

```bash
git clone https://github.com/cutiechi/k8s-urls.git
cd k8s-urls
cargo install --path .
```

## Usage

```bash
# List all services in default namespace
k8s-urls

# List services in a specific namespace
k8s-urls -n kube-system

# Filter services by name using regex
k8s-urls -f "nginx.*"

# Use a specific kubeconfig file
k8s-urls -k /path/to/kubeconfig

# Combine options
k8s-urls -n production -f ".*-api$"
```

### Command Line Options

- `-n, --namespace`: Specify Kubernetes namespace (default: "default")
- `-k, --kubeconfig`: Optional path to kubeconfig file
- `-f, --filter`: Filter services by name using regex pattern

## Output Format

For each service, the tool displays:
- Service DNS names
- ClusterIP URLs (if applicable)
- Pod IP URLs
- External access points (if available)
- Headless service DNS records (if applicable)

Example output:
```
=== 命名空间: default ===

服务: my-service
服务 DNS: my-service.default.svc.cluster.local
类型: ClusterIP Service
  ClusterIP URL: http://10.96.0.1:80 (http)
  DNS URL: http://my-service.default.svc.cluster.local:80 (http)
Pod 端点:
  Pod: my-pod-1
    IP URL: http://10.244.0.1:80
    DNS URL: http://my-pod-1.my-service.default.svc.cluster.local:80
```

## Dependencies

- `tokio`: Async runtime
- `kube`: Kubernetes client
- `k8s-openapi`: Kubernetes API types
- `clap`: Command line argument parsing
- `regex`: Service name filtering

## Requirements

- Valid Kubernetes cluster access
- Properly configured kubeconfig file
- Appropriate RBAC permissions to list services and endpoints

## License

MIT License
