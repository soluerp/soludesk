# Política de Privacidade do SOLUDESK

*Última atualização: 02/11/2026*

O SOLUDESK é um aplicativo de suporte remoto supervisionado, desenvolvido pela SOLUERP, para Windows e Android. Esta política explica quais dados o aplicativo acessa, para quê e o que acontece com eles.

## Resumo

- O SOLUDESK **não coleta, não vende e não compartilha** dados pessoais com terceiros.
- O aplicativo **não tem anúncios, telemetria nem ferramentas de análise**.
- Uma sessão de acesso remoto só começa com a **senha do aparelho atendido** e o **aceite visível de quem está nele**, e pode ser encerrada por ele a qualquer momento.

## Dados acessados e por quê

| Dado ou permissão | Para que é usado | Quando |
|---|---|---|
| **Identificador do aparelho (ID SOLUDESK)** e endereço de rede | Registrar o aparelho no servidor SOLUDESK e estabelecer a conexão entre os dois lados | Enquanto o app estiver aberto ou com o serviço ativo |
| **Conteúdo da tela** (captura de tela do Android) | Mostrar a tela do celular ao técnico durante a sessão | Só durante uma sessão aceita, depois que o usuário inicia o compartilhamento e autoriza a captura no aviso do Android |
| **Serviço de acessibilidade ("SOLUDESK Input")** | Executar no celular os toques, gestos e textos enviados pelo técnico durante a sessão | Só durante uma sessão aceita, e só se o usuário ativar esse serviço nas configurações do Android |
| **Microfone / áudio** | Transmitir o som do aparelho durante a sessão, se a pessoa permitir | Só durante uma sessão aceita |
| **Arquivos escolhidos** | Transferir arquivos entre os aparelhos quando a pessoa pedir | Só os arquivos que a pessoa selecionar |
| **Área de transferência** | Copiar e colar texto entre os aparelhos durante a sessão | Só durante uma sessão aceita |
| **Notificações e execução em segundo plano** | Mostrar que o compartilhamento está ativo e manter a sessão funcionando | Enquanto o compartilhamento estiver ativo |

O serviço de acessibilidade **não é usado para ler, guardar ou enviar** o conteúdo de outros aplicativos, nem para nenhuma finalidade além do controle remoto pedido e aceito pelo usuário.

## Para onde os dados vão

- A conexão usa o servidor SOLUDESK (`vps72894.publiccloud.com.br`), operado pela SOLUERP, para localizar os aparelhos e, quando a conexão direta não é possível, retransmitir o tráfego.
- O conteúdo da sessão (imagem, áudio, comandos, arquivos) é **criptografado de ponta a ponta** entre os dois aparelhos. O servidor não consegue ver nem guarda esse conteúdo.
- O servidor guarda apenas o necessário para funcionar: o ID e a chave pública de cada aparelho e registros técnicos de conexão (endereço IP e horário), usados para segurança e diagnóstico.
- Nada é enviado para servidores do RustDesk nem de terceiros.

## Retenção e exclusão

- Os dados da sessão não são armazenados: existem só enquanto a sessão dura.
- Os registros técnicos do servidor são mantidos pelo tempo necessário para segurança e diagnóstico.
- Para pedir a exclusão do registro do seu aparelho, desinstale o aplicativo e entre em contato pelo e-mail abaixo informando o ID SOLUDESK.

## Crianças

O SOLUDESK é uma ferramenta de suporte técnico e não é destinado a menores de 13 anos.

## Código aberto

O SOLUDESK é derivado do RustDesk e distribuído sob a licença AGPL-3.0. O código-fonte completo está em https://github.com/soluerp/soludesk.

## Contato

SOLUERP — suporte@solucoesetecnologia.com.br

---

# SOLUDESK Privacy Policy (English summary)

SOLUDESK is an attended remote-support app by SOLUERP. It collects no personal data for advertising or analytics and shares nothing with third parties. A remote session only starts with the device password and visible acceptance by the person at the device. During an accepted session only, the app uses Android screen capture (to show the screen), the "SOLUDESK Input" accessibility service if the user enables it (to perform the remote technician's taps and gestures), audio (if allowed), user-selected files and the clipboard. The accessibility service is not used to read, store or send content from other apps. Session content is end-to-end encrypted between the two devices; the SOLUDESK server (`vps72894.publiccloud.com.br`) only brokers/relays connections and keeps device IDs, public keys and technical connection logs. Contact: suporte@solucoesetecnologia.com.br.
