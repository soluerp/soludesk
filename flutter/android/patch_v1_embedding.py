#!/usr/bin/env python3
"""Faz plugins Flutter antigos compilarem no Flutter 3.29+ (Android).

O Flutter removeu PluginRegistry.Registrar (embedding v1). Plugins travados em
versões antigas no pubspec.lock (para manter o build Windows igual) ainda
referenciam esse tipo no registerWith(), que o app nunca chama (o
GeneratedPluginRegistrant usa o embedding v2).

Para cada plugin Android listado em .flutter-plugins-dependencies, os arquivos
.java/.kt que citam o Registrar passam a usar uma interface local com a mesma
API (V1Registrar), criada no pacote do arquivo. Só roda no CI do Android.
"""
import json
import pathlib
import re
import sys

STUB = """package {pkg};

import android.app.Activity;
import android.content.Context;
import io.flutter.plugin.common.BinaryMessenger;
import io.flutter.plugin.common.PluginRegistry;
import io.flutter.plugin.platform.PlatformViewRegistry;
import io.flutter.view.TextureRegistry;

// Substituto de PluginRegistry.Registrar (embedding v1, removido do Flutter).
// Nunca é instanciado: existe só para o código legado compilar.
public interface V1Registrar {{
  Activity activity();
  Context context();
  Context activeContext();
  BinaryMessenger messenger();
  TextureRegistry textures();
  PlatformViewRegistry platformViewRegistry();
  String lookupKeyForAsset(String asset);
  String lookupKeyForAsset(String asset, String packageName);
  V1Registrar publish(Object value);
  V1Registrar addRequestPermissionsResultListener(PluginRegistry.RequestPermissionsResultListener listener);
  V1Registrar addActivityResultListener(PluginRegistry.ActivityResultListener listener);
  V1Registrar addNewIntentListener(PluginRegistry.NewIntentListener listener);
  V1Registrar addUserLeaveHintListener(PluginRegistry.UserLeaveHintListener listener);
  V1Registrar addWindowFocusChangedListener(PluginRegistry.WindowFocusChangedListener listener);
}}
"""

FQ = "io.flutter.plugin.common.PluginRegistry.Registrar"


def patch_file(path: pathlib.Path) -> bool:
    text = path.read_text(encoding="utf-8")
    if "Registrar" not in text:
        return False
    m = re.search(r"^\s*package\s+([\w.]+)\s*;?", text, re.M)
    if not m:
        print(f"  sem package, ignorado: {path}")
        return False
    pkg = m.group(1)
    orig = text
    kotlin = path.suffix == ".kt"
    imported = re.search(r"^import\s+io\.flutter\.plugin\.common\.PluginRegistry\.Registrar\s*;?\s*$", text, re.M)
    text = re.sub(r"^import\s+io\.flutter\.plugin\.common\.PluginRegistry\.Registrar\s*;?\s*$\n?", "", text, flags=re.M)
    text = text.replace(FQ, "V1Registrar")
    text = re.sub(r"\bPluginRegistry\.Registrar\b", "V1Registrar", text)
    if imported:
        text = re.sub(r"(?<![\w.])Registrar\b", "V1Registrar", text)
    if text == orig:
        return False
    path.write_text(text, encoding="utf-8")
    stub = path.parent / "V1Registrar.java"
    if not stub.exists():
        stub.write_text(STUB.format(pkg=pkg), encoding="utf-8")
    print(f"  ajustado: {path}{' (kotlin)' if kotlin else ''}")
    return True


def main() -> int:
    deps_file = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".flutter-plugins-dependencies")
    deps = json.loads(deps_file.read_text(encoding="utf-8"))
    total = 0
    for plugin in deps.get("plugins", {}).get("android", []):
        src = pathlib.Path(plugin["path"]) / "android" / "src"
        if not src.is_dir():
            continue
        files = [f for f in src.rglob("*") if f.suffix in (".java", ".kt") and f.name != "V1Registrar.java"]
        changed = [f for f in files if patch_file(f)]
        if changed:
            print(f"{plugin['name']}: {len(changed)} arquivo(s)")
            total += len(changed)
    print(f"Total ajustado: {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
