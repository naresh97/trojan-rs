use std::net::IpAddr;

use anyhow::anyhow;
use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};

#[derive(Clone)]
pub struct DnsResolver {
    resolver: hickory_resolver::AsyncResolver<
        hickory_resolver::name_server::GenericConnector<
            hickory_resolver::name_server::TokioRuntimeProvider,
        >,
    >,
}

impl DnsResolver {
    pub async fn new() -> DnsResolver {
        let resolver =
            TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
        DnsResolver { resolver }
    }
    pub async fn resolve(&self, domain_name: &str) -> anyhow::Result<IpAddr> {
        let response = self.resolver.lookup_ip(domain_name).await?;
        let ip = response
            .iter()
            .next()
            .ok_or(anyhow!("No IP addresses found"))?;
        Ok(ip)
    }
}
