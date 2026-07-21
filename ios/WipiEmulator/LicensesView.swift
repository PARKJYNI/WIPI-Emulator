// 오픈소스 고지 화면. MIT 라이선스는 저작권 고지 포함이 의무라서
// 각 컴포넌트의 저작권자와 라이선스 원문을 그대로 표시한다.

import SwiftUI

private struct LicenseEntry: Identifiable {
    let name: String
    let roleKey: LocalizedStringKey
    let text: String
    var id: String { name }
}

private func mit(_ copyright: String) -> String {
    """
    \(copyright)

    Permission is hereby granted, free of charge, to any person obtaining a copy \
    of this software and associated documentation files (the "Software"), to deal \
    in the Software without restriction, including without limitation the rights \
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell \
    copies of the Software, and to permit persons to whom the Software is \
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in \
    all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR \
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, \
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE \
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER \
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, \
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN \
    THE SOFTWARE.
    """
}

private let entries: [LicenseEntry] = [
    LicenseEntry(
        name: "wie",
        roleKey: "license_role_wie",
        text: mit("Copyright 2020 Inseok Lee")
    ),
    LicenseEntry(
        name: "RustJava",
        roleKey: "license_role_rustjava",
        text: mit("Copyright 2020 Inseok Lee")
    ),
    LicenseEntry(
        name: "smaf",
        roleKey: "license_role_smaf",
        text: mit("Copyright 2020 Inseok Lee")
    ),
    LicenseEntry(
        name: "rodio / cpal",
        roleKey: "license_role_rodio",
        text: mit("Copyright (c) The Rodio Project Contributors") + """


        ---

        cpal is licensed under the Apache License, Version 2.0. \
        You may obtain a copy of the License at:
        http://www.apache.org/licenses/LICENSE-2.0
        """
    ),
    LicenseEntry(
        name: "rustysynth",
        roleKey: "license_role_rustysynth",
        text: mit("Copyright (c) 2021 Nobuaki Tanaka")
    ),
    LicenseEntry(
        name: "GeneralUser GS",
        roleKey: "license_role_soundfont",
        text: """
        GeneralUser GS by S. Christian Collins
        (schristiancollins.com/generaluser.php)

        Licensed under the GeneralUser GS License v2.0: free to use, modify \
        and distribute, including in commercial software, with attribution \
        appreciated. No warranty is provided.
        """
    ),
]

struct LicensesView: View {
    var body: some View {
        List {
            Section {
                Text("licenses_credit")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }

            Section {
                ForEach(entries) { entry in
                    NavigationLink {
                        ScrollView {
                            Text(entry.text)
                                .font(.system(.caption, design: .monospaced))
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .padding()
                        }
                        .navigationTitle(entry.name)
                        .navigationBarTitleDisplayMode(.inline)
                    } label: {
                        VStack(alignment: .leading, spacing: 2) {
                            Text(entry.name)
                            Text(entry.roleKey)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            } footer: {
                Text("그 외 zip, tracing 등 다수의 Rust 크레이트(MIT/Apache-2.0)가 포함되어 있습니다. 게임 파일은 이 앱에 포함되어 있지 않으며, 사용자가 소유한 파일만 불러올 수 있습니다.")
            }
        }
        .navigationTitle("licenses_title")
        .navigationBarTitleDisplayMode(.inline)
    }
}
