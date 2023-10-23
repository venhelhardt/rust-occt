#pragma once

#include <memory>

namespace occt {

struct MeshVerts;
struct MeshNorms;
struct MeshTris;
struct MeshBbox;
struct MeshBlobInit;

struct MeshBlob
{
    MeshBlob() noexcept;
    MeshBlob(MeshBlobInit) noexcept;
    MeshBlob(const MeshBlob&) = delete;

    ~MeshBlob() noexcept;

    MeshBbox bbox() const noexcept;
    MeshVerts verts() const noexcept;
    MeshNorms norms() const noexcept;
    MeshTris tris() const noexcept;

    MeshBlob& operator=(const MeshBlob&) = delete;

private:
    struct Impl;

    Impl* m_d;
};

std::unique_ptr<MeshBlob> make_flask(double width, double thickness, double height) noexcept;

} // namespace occt