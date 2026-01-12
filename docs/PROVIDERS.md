## DNS Provider Matrix

### Supported Providers

| Provider                  | Supported? | Sandbox? | Author | Maintainer | Notes                                                                                |
|---------------------------|:----------:|:--------:|--------|------------|--------------------------------------------------------------------------------------|
| Cloudflare                | ✅         | ❌       | tarka  | tarka      | Tested against personal account                                                      |
| deSEC.io                  | ✅         | ❌       | tarka  | tarka      | Tested against personal account                                                      |
| DnSimple                  | ✅         | ✅       | tarka  | tarka      |                                                                                      |
| Gandi                     | ✅         | ‼️        | tarka  |  tarka     | Tested against personal account. (Has a sandbox but is unusable in practice?)       |
| DNSMadeEasy               | ✅         | ✅       | tarka  | tarka      | Sandbox uses legacy ciphers and fails with rustls.                                   |
| Porkbun                   | ✅         | ❌       | tarka  | tarka      | Tested against personal account                                                      |

### Unsupported Providers

The following list is generated from [acme.sh](https://acme.sh) and may not be
entirely accurate. Corrections and updates welcome.

| Provider                  | Supported? | Sandbox? | Author | Maintainer | Notes                                                                                |
|---------------------------|:----------:|:--------:|--------|------------|--------------------------------------------------------------------------------------|
| 1984Hosting               | ❌         | ❌       |        |            | Doesn't really have an API, but the web forms can be abused. See acme.sh.            |
| ACME-DNS                  | ❌         | ✅       |        |            | Self-hosted DNS shim for DNS-01 support. Run locally for testing.                    |
| AcmeProxy.pl              | ❌         | ✅       |        |            | Similar to ACME-DNS.                                                                 |
| Active24                  | ❌         | ✅       |        |            |                                                                                      |
| Akamai                    | ❌         | ❔       |        |            |                                                                                      |
| Aliyun                    | ❌         | ❔       |        |            |                                                                                      |
| All-Inkl                  | ❌         | ❔       |        |            |                                                                                      |
| Alviy.com                 | ❌         | ❔       |        |            |                                                                                      |
| Alwaysdata                | ❌         | ❔       |        |            |                                                                                      |
| Amazon                    | ❌         | ❔       |        |            |                                                                                      |
| anexia.com                | ❌         | ❔       |        |            |                                                                                      |
| Anikeen                   | ❌         | ❔       |        |            |                                                                                      |
| ArtFiles.de               | ❌         | ❔       |        |            |                                                                                      |
| Aruba                     | ❌         | ❔       |        |            |                                                                                      |
| ArvanCloud                | ❌         | ❔       |        |            |                                                                                      |
| autoDNS                   | ❌         | ❔       |        |            |                                                                                      |
| Azure                     | ❌         | ❔       |        |            |                                                                                      |
| Beget.com                 | ❌         | ❔       |        |            |                                                                                      |
| bookmyname                | ❌         | ❔       |        |            |                                                                                      |
| Bunny                     | ❌         | ❔       |        |            |                                                                                      |
| CloudDNS                  | ❌         | ❔       |        |            |                                                                                      |
| ClouDNS.net               | ❌         | ❔       |        |            |                                                                                      |
| ConoHa                    | ❌         | ❔       |        |            |                                                                                      |
| Constellix                | ❌         | ❔       |        |            |                                                                                      |
| cPanel                    | ❌         | ❔       |        |            |                                                                                      |
| DDNSS.de                  | ❌         | ❔       |        |            |                                                                                      |
| DigitalOcean              | ❌         | ❔       |        |            |                                                                                      |
| DirectAdmin               | ❌         | ❔       |        |            |                                                                                      |
| dns.la                    | ❌         | ❔       |        |            |                                                                                      |
| DNS.Services              | ❌         | ❔       |        |            |                                                                                      |
| DNSExit                   | ❌         | ❔       |        |            |                                                                                      |
| dnsHome.de                | ❌         | ❔       |        |            |                                                                                      |
| DNSPod.com                | ❌         | ❔       |        |            |                                                                                      |
| do.de                     | ❌         | ❔       |        |            |                                                                                      |
| Domeneshop                | ❌         | ❔       |        |            |                                                                                      |
| DreamHost                 | ❌         | ❔       |        |            |                                                                                      |
| DuckDNS.org               | ❌         | ❔       |        |            |                                                                                      |
| durabledns                | ❌         | ❔       |        |            |                                                                                      |
| Dyn                       | ❌         | ❔       |        |            |                                                                                      |
| dynadot                   | ❌         | ❔       |        |            | API is only for purchase/sale of domains?                                            |
| dyndnsfree                | ❌         | ❔       |        |            |                                                                                      |
| Dynu                      | ❌         | ❔       |        |            |                                                                                      |
| dynv6                     | ❌         | ❔       |        |            |                                                                                      |
| easyDNS.net               | ❌         | ✅       |        |            |                                                                                      |
| Euserv.eu                 | ❌         | ❔       |        |            |                                                                                      |
| Exoscale                  | ❌         | ❔       |        |            |                                                                                      |
| fornex.com                | ❌         | ❔       |        |            |                                                                                      |
| FreeDNS                   | ❌         | ❔       |        |            |                                                                                      |
| Futurehosting             | ❌         | ❔       |        |            |                                                                                      |
| GCore                     | ❌         | ❔       |        |            |                                                                                      |
| Geoscaling                | ❌         | ❔       |        |            |                                                                                      |
| GoDaddy.com               | ❌         | ❔       |        |            |                                                                                      |
| Google                    | ❌         | ❔       |        |            |                                                                                      |
| HE                        | ❌         | ❔       |        |            |                                                                                      |
| Hetzner                   | ❌         | ❔       |        |            |                                                                                      |
| hexonet.com               | ❌         | ❔       |        |            |                                                                                      |
| hosting.de                | ❌         | ❔       |        |            |                                                                                      |
| HostingUkraine            | ❌         | ❔       |        |            |                                                                                      |
| Hosttech                  | ❌         | ❔       |        |            |                                                                                      |
| HuaweiCloud               | ❌         | ❔       |        |            |                                                                                      |
| Hurricane                 | ❌         | ❔       |        |            |                                                                                      |
| Infoblox                  | ❌         | ❔       |        |            |                                                                                      |
| infomaniak                | ❌         | ❔       |        |            |                                                                                      |
| internetbs                | ❌         | ❔       |        |            |                                                                                      |
| INWX                      | ❌         | ❔       |        |            |                                                                                      |
| IPv64                     | ❌         | ❔       |        |            |                                                                                      |
| ISPConfig                 | ❌         | ❔       |        |            |                                                                                      |
| ISPMan                    | ❌         | ❔       |        |            |                                                                                      |
| jdcloud.com               | ❌         | ❔       |        |            |                                                                                      |
| Joker.com                 | ❌         | ❔       |        |            |                                                                                      |
| kapper.net                | ❌         | ❔       |        |            |                                                                                      |
| KingHost                  | ❌         | ❔       |        |            |                                                                                      |
| Leaseweb.com              | ❌         | ❔       |        |            |                                                                                      |
| Lima-City                 | ❌         | ❔       |        |            |                                                                                      |
| Linode                    | ❌         | ❔       |        |            |                                                                                      |
| Loopia                    | ❌         | ❔       |        |            |                                                                                      |
| LuaDNS                    | ❌         | ❔       |        |            |                                                                                      |
| MailinaBox                | ❌         | ❔       |        |            |                                                                                      |
| MaraDNS                   | ❌         | ❔       |        |            |                                                                                      |
| mijn.host                 | ❌         | ❔       |        |            |                                                                                      |
| Misaka.io                 | ❌         | ❔       |        |            |                                                                                      |
| MyDevil.net               | ❌         | ❔       |        |            |                                                                                      |
| myLoc.de                  | ❌         | ❔       |        |            |                                                                                      |
| mythic-beasts.com         | ❌         | ❔       |        |            |                                                                                      |
| Name.com                  | ❌         | ❔       |        |            |                                                                                      |
| Namecheap                 | ❌         | ✅       |        |            | API is poorly documented and dangerous (see discussion on API pages).                |
| Namemaster                | ❌         | ❔       |        |            |                                                                                      |
| Nanelo                    | ❌         | ❔       |        |            |                                                                                      |
| Neodigit.net              | ❌         | ❔       |        |            |                                                                                      |
| Netcup                    | ❌         | ❔       |        |            |                                                                                      |
| Netlify                   | ❌         | ❔       |        |            |                                                                                      |
| Nexcess                   | ❌         | ❔       |        |            |                                                                                      |
| Njalla                    | ❌         | ❔       |        |            |                                                                                      |
| NLnetLabs                 | ❌         | ❔       |        |            |                                                                                      |
| Nodion                    | ❌         | ❔       |        |            |                                                                                      |
| NS1.com                   | ❌         | ❔       |        |            |                                                                                      |
| Online                    | ❌         | ❔       |        |            |                                                                                      |
| OpenProvider              | ❌         | ❔       |        |            |                                                                                      |
| OpenStack                 | ❌         | ❔       |        |            |                                                                                      |
| OPNsense                  | ❌         | ❔       |        |            |                                                                                      |
| Oracle                    | ❌         | ❔       |        |            |                                                                                      |
| pdd.yandex.ru             | ❌         | ❔       |        |            |                                                                                      |
| PDNS                      | ❌         | ❔       |        |            |                                                                                      |
| Plesk                     | ❌         | ❔       |        |            |                                                                                      |
| PointHQ                   | ❌         | ❔       |        |            |                                                                                      |
| RackCorp                  | ❌         | ❔       |        |            |                                                                                      |
| rage4                     | ❌         | ❔       |        |            |                                                                                      |
| reg.ru                    | ❌         | ❔       |        |            |                                                                                      |
| s-dns.de                  | ❌         | ❔       |        |            |                                                                                      |
| Scaleway                  | ❌         | ❔       |        |            |                                                                                      |
| Schlundtech               | ❌         | ❔       |        |            |                                                                                      |
| selectel.com(selectel.ru) | ❌         | ❔       |        |            |                                                                                      |
| Selfhost                  | ❌         | ❔       |        |            |                                                                                      |
| Shellrent                 | ❌         | ❔       |        |            |                                                                                      |
| Simply.com                | ❌         | ❔       |        |            |                                                                                      |
| Synology                  | ❌         | ❔       |        |            |                                                                                      |
| TencentCloud              | ❌         | ❔       |        |            |                                                                                      |
| Thermo.io                 | ❌         | ❔       |        |            |                                                                                      |
| Timeweb                   | ❌         | ❔       |        |            |                                                                                      |
| TransIP                   | ❌         | ❔       |        |            |                                                                                      |
| UltraDNS                  | ❌         | ❔       |        |            |                                                                                      |
| variomedia.de             | ❌         | ❔       |        |            |                                                                                      |
| Veesp                     | ❌         | ❔       |        |            |                                                                                      |
| Vercel                    | ❌         | ❔       |        |            |                                                                                      |
| VSCALE                    | ❌         | ❔       |        |            |                                                                                      |
| Vultr                     | ❌         | ❔       |        |            |                                                                                      |
| Websupport                | ❌         | ❔       |        |            |                                                                                      |
| WEDOS                     | ❌         | ❔       |        |            |                                                                                      |
| Wedos                     | ❌         | ❔       |        |            |                                                                                      |
| West.cn                   | ❌         | ❔       |        |            |                                                                                      |
| World4You                 | ❌         | ❔       |        |            |                                                                                      |
| Yandex                    | ❌         | ❔       |        |            |                                                                                      |
| Zilore                    | ❌         | ❔       |        |            |                                                                                      |
| ZoneEdit                  | ❌         | ❔       |        |            |                                                                                      |
| zonomi.com                | ❌         | ❔       |        |            |                                                                                      |
