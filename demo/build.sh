#!/bin/sh
# 데모 게임 빌드: javac(--release 8, RustJava classfile 45~70 지원 범위) + jar.
# 사용: ./build.sh → demo.jar 생성. 검증: rust/에서
#   cargo run --release -p wipi_android --bin headless -- ../demo/demo.jar /tmp/demo 5
set -e
cd "$(dirname "$0")"

JAVAC="${JAVA_HOME:-/Applications/Android Studio.app/Contents/jbr/Contents/Home}/bin/javac"
JAR="${JAVA_HOME:-/Applications/Android Studio.app/Contents/jbr/Contents/Home}/bin/jar"

rm -rf build demo.jar
mkdir -p build

"$JAVAC" --release 8 -encoding UTF-8 -cp stubs -d build $(find stubs src -name "*.java")
# 스텁 클래스는 jar에서 제외 (런타임엔 에뮬레이터가 제공)
rm -rf build/javax

cat > build/MANIFEST.MF << 'EOF'
Manifest-Version: 1.0
MIDlet-Name: WipiDemo
MIDlet-Version: 1.0.0
MIDlet-Vendor: ParkJeongseop
MIDlet-1: Heart Catch, , DemoGame
MicroEdition-Profile: MIDP-2.0
MicroEdition-Configuration: CLDC-1.1
EOF

# assets(big.icon=표지, __adf__=EUC-KR 게임명)를 jar 루트에 포함 —
# 임포트 시 라이브러리가 표지/이름을 추출한다. KTF/LGT/SKT jar 감지에는 영향 없음(각각 client.bin/binary.mod/매직바이트 검사).
"$JAR" cfm demo.jar build/MANIFEST.MF -C build . -C assets .
echo "OK: $(pwd)/demo.jar"
