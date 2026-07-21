// 게임 라이브러리 화면 — 표지 그리드. 탭하면 실행, 롱프레스로 삭제, + 로 임포트.

import SwiftUI
import UniformTypeIdentifiers

struct LibraryView: View {
    @ObservedObject var library: GameLibrary
    let onPlay: (GameEntry) -> Void
    let runError: String?
    let onDismissError: () -> Void

    @State private var showPicker = false
    @State private var showSettings = false
    @State private var importError: String?

    private let columns = [GridItem(.adaptive(minimum: 100), spacing: 16)]

    var body: some View {
        NavigationStack {
            Group {
                if library.games.isEmpty {
                    emptyState
                } else {
                    grid
                }
            }
            .navigationTitle("app_name")
            .safeAreaInset(edge: .bottom) { errorBanner }
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button { showSettings = true } label: { Image(systemName: "gearshape") }
                }
                ToolbarItem(placement: .topBarTrailing) {
                    Button { showPicker = true } label: { Image(systemName: "plus") }
                }
            }
            .sheet(isPresented: $showSettings) { SettingsView() }
            .fileImporter(
                isPresented: $showPicker,
                allowedContentTypes: [.zip, UTType(filenameExtension: "jar") ?? .data],
                allowsMultipleSelection: false
            ) { handleImport($0) }
            .alert("import_failed_title", isPresented: .constant(importError != nil)) {
                Button("action_ok") { importError = nil }
            } message: {
                Text(importError ?? "")
            }
        }
    }

    private var grid: some View {
        ScrollView {
            LazyVGrid(columns: columns, spacing: 16) {
                ForEach(library.games) { game in
                    GameCell(game: game)
                        .onTapGesture { onPlay(game) }
                        .contextMenu {
                            Button(role: .destructive) {
                                library.delete(game)
                            } label: {
                                Label("action_delete", systemImage: "trash")
                            }
                        }
                }
            }
            .padding()
        }
    }

    private var emptyState: some View {
        VStack(spacing: 12) {
            Image(systemName: "gamecontroller")
                .font(.system(size: 48))
                .foregroundStyle(.secondary)
            Text("library_empty_title")
                .font(.headline)
            Text("library_empty_hint")
                .font(.subheadline)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
        }
        .padding()
    }

    @ViewBuilder
    private var errorBanner: some View {
        if let runError {
            Text(runError)
                .font(.footnote)
                .foregroundStyle(.white)
                .padding(12)
                .frame(maxWidth: .infinity)
                .background(.red)
                .onTapGesture { onDismissError() }
        }
    }

    private func handleImport(_ result: Result<[URL], Error>) {
        switch result {
        case .success(let urls):
            guard let url = urls.first else { return }
            do {
                try library.importGame(from: url)
            } catch {
                importError = error.localizedDescription
            }
        case .failure(let error):
            importError = error.localizedDescription
        }
    }
}

private struct GameCell: View {
    let game: GameEntry

    var body: some View {
        VStack(spacing: 6) {
            ZStack {
                RoundedRectangle(cornerRadius: 12)
                    .fill(Color(.secondarySystemBackground))
                if let cover = game.cover {
                    Image(uiImage: cover)
                        .resizable()
                        .interpolation(.none) // 저해상도 아이콘을 nearest로 확대(픽셀아트 느낌)
                        .aspectRatio(contentMode: .fit)
                        .padding(8)
                } else {
                    Image(systemName: "gamecontroller")
                        .font(.largeTitle)
                        .foregroundStyle(.secondary)
                }
            }
            .aspectRatio(1, contentMode: .fit)

            Text(game.name)
                .font(.caption)
                .lineLimit(2)
                .multilineTextAlignment(.center)
                .frame(height: 32, alignment: .top)
        }
    }
}
