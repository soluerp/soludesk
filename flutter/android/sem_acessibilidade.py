#!/usr/bin/env python3
"""Versão do SOLUDESK sem o serviço de Acessibilidade.

No Brasil, o Play Protect bloqueia apps recebidos por navegador, WhatsApp ou
gerenciador de arquivos que declaram um serviço de Acessibilidade. Esta versão
remove o InputService do manifesto (o código Kotlin já trata o serviço ausente:
InputService.ctx fica nulo) para o APK poder ser instalado direto.

  sem_acessibilidade.py remover <AndroidManifest.xml>
  sem_acessibilidade.py conferir <arquivo.apk>
"""
import re
import sys
import zipfile


def remover(manifest: str) -> None:
    text = open(manifest, encoding="utf-8").read()
    padrao = re.compile(r'\n[ \t]*<service\s+android:name="\.InputService".*?</service>[ \t]*', re.S)
    novo, n = padrao.subn("", text)
    if n != 1:
        sys.exit(f"esperava 1 bloco do InputService no manifesto, achei {n}")
    if "BIND_ACCESSIBILITY_SERVICE" in novo:
        sys.exit("o manifesto ainda cita BIND_ACCESSIBILITY_SERVICE")
    open(manifest, "w", encoding="utf-8").write(novo)
    print("InputService removido do manifesto")


def conferir(apk: str) -> None:
    with zipfile.ZipFile(apk) as z:
        dados = z.read("AndroidManifest.xml")
    texto = dados.decode("utf-16-le", errors="ignore") + dados.decode("utf-8", errors="ignore")
    for proibido in ("BIND_ACCESSIBILITY_SERVICE", "android.accessibilityservice"):
        if proibido in texto:
            sys.exit(f"{apk}: o manifesto ainda contém {proibido}")
    print(f"{apk}: sem serviço de Acessibilidade")


if __name__ == "__main__":
    if len(sys.argv) != 3 or sys.argv[1] not in ("remover", "conferir"):
        sys.exit(__doc__)
    {"remover": remover, "conferir": conferir}[sys.argv[1]](sys.argv[2])
