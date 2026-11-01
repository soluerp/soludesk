# SOLUDESK

**Acesso remoto simples, rápido e seguro** · *Simple, fast and secure remote support*

SOLUDESK é um cliente Windows de suporte remoto supervisionado, derivado do [RustDesk](https://github.com/rustdesk/rustdesk) e distribuído sob a mesma licença **AGPL-3.0**. Este repositório contém o código-fonte completo correspondente aos binários publicados.

*SOLUDESK is an attended remote-support client for Windows, derived from [RustDesk](https://github.com/rustdesk/rustdesk) and distributed under the same **AGPL-3.0** license. This repository holds the complete corresponding source code of every published binary.*

## O que muda em relação ao RustDesk / *Changes from upstream RustDesk*

- Conecta somente ao servidor SOLUDESK configurado no código; sem fallback para servidores públicos. / *Connects only to the SOLUDESK server built into the code; no fallback to public servers.*
- Cada sessão exige a senha do computador atendido e um aceite visível de quem está nele. / *Every session requires the remote computer's password and a visible acceptance by the person at that computer.*
- Elevação remota e contorno do UAC removidos. / *Remote elevation and UAC bypass removed.*
- Verificação de atualizações do RustDesk desativada (cliente personalizado). / *RustDesk update checks disabled (custom client).*
- Marca, cores e textos em português do Brasil. / *Branding, colors and Brazilian Portuguese texts.*

## Compilação / *Build*

Os binários oficiais são compilados pelo GitHub Actions a partir deste repositório: workflow [`.github/workflows/soludesk-windows.yml`](.github/workflows/soludesk-windows.yml) (Windows x64, Flutter 3.47.7, Rust 1.99.1, vcpkg com `vcpkg.json`).

*Official binaries are built by GitHub Actions from this repository using the workflow above.*

Submódulo / *Submodule*: `libs/hbb_common` → [soluerp/hbb_common](https://github.com/soluerp/hbb_common).

## Code signing policy

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by [SignPath Foundation](https://signpath.org).

- **Committers and reviewers:** [soluerp](https://github.com/soluerp)
- **Approvers:** [soluerp](https://github.com/soluerp)

Only binaries built by the GitHub Actions workflow of this repository, from commits on the release branch, are submitted for signing. Every signing request is approved manually by an approver.

## Privacy policy

This program will not transfer any information to other networked systems unless specifically requested by the user or the person installing or operating it.

To provide remote support, SOLUDESK connects to the SOLUDESK ID/relay server built into the application (`vps72894.publiccloud.com.br`) to register the device ID and to establish sessions that the users start and accept. Session traffic between the two computers is encrypted. The application includes no telemetry or analytics service, and RustDesk's update check is disabled.

## Licença / *License*

AGPL-3.0 — ver / *see* [LICENCE](LICENCE). Copyright dos autores do RustDesk (Purslane Tech Pte. Ltd. e colaboradores) e das alterações do SOLUDESK. / *Copyright of the RustDesk authors (Purslane Tech Pte. Ltd. and contributors) and of the SOLUDESK changes.*

"RustDesk" é marca de seus titulares; o SOLUDESK não é afiliado nem endossado pelo projeto RustDesk. / *"RustDesk" is a trademark of its owners; SOLUDESK is not affiliated with or endorsed by the RustDesk project.*
