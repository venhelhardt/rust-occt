#include "occt/src/occt.h"

#include <cstdint>
#include <vector>

#include "occt/src/occt.rs.h"

#include <BRepAlgoAPI_Fuse.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepMesh_DiscretFactory.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepTools.hxx>
#include <GC_MakeArcOfCircle.hxx>
#include <GC_MakeSegment.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <gp_Ax1.hxx>
#include <gp_Pnt.hxx>
#include <TopExp_Explorer.hxx>
#include <TopLoc_Location.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>

namespace occt {

namespace {

struct Vertex
{
    float x, y, z;
};

struct Triangle
{
    std::uint32_t x, y, z;
};

} // namespace

struct MeshBlobInit
{
    std::vector<Vertex> verts;
    std::vector<Vertex> norms;
    std::vector<Triangle> tris;
};

struct MeshBlob::Impl
{
    Impl()
    {
        bbox.min = {0.0f, 0.0f, 0.0f};
        bbox.max = {0.0f, 0.0f, 0.0f};
    }

    std::vector<Vertex> verts;
    std::vector<Vertex> norms;
    std::vector<Triangle> tris;
    MeshBbox bbox;
};

MeshBlob::MeshBlob() noexcept
  : m_d(new Impl())
{
    m_d->verts.push_back({-0.5f, -0.5f, 0.0f});
    m_d->verts.push_back({0.5f, -0.5f, 0.0f});
    m_d->verts.push_back({0.0f, 0.5f, 0.0f});

    m_d->norms.push_back({0.0f, 0.0f, 1.0f});
    m_d->norms.push_back({0.0f, 0.0f, 1.0f});
    m_d->norms.push_back({0.0f, 0.0f, 1.0f});

    m_d->tris.push_back({0, 1, 2});
}

MeshBlob::MeshBlob(MeshBlobInit init) noexcept
  : m_d(new Impl())
{
    m_d->verts = std::move(init.verts);
    m_d->norms = std::move(init.norms);
    m_d->tris = std::move(init.tris);

    if(!m_d->verts.empty())
    {
        Tuple3f min = {m_d->verts[0].x, m_d->verts[0].y, m_d->verts[0].z};
        Tuple3f max = min;

        for(const Vertex& v : m_d->verts)
        {
            if(min.x > v.x)
            {
                min.x = v.x;
            }

            if(min.y > v.y)
            {
                min.y = v.y;
            }

            if(min.z > v.z)
            {
                min.z = v.z;
            }

            if(max.x < v.x)
            {
                max.x = v.x;
            }

            if(max.y < v.y)
            {
                max.y = v.y;
            }

            if(max.z < v.z)
            {
                max.z = v.z;
            }
        }

        m_d->bbox.min = min;
        m_d->bbox.max = max;
    }
}

MeshBlob::~MeshBlob() noexcept
{
    delete m_d;
}

MeshBbox MeshBlob::bbox() const noexcept
{
    return m_d->bbox;
}

MeshVerts MeshBlob::verts() const noexcept
{
    MeshVerts raw;

    raw.count = static_cast<std::uint32_t>(m_d->verts.size());
    raw.ptr = reinterpret_cast<const float*>(m_d->verts.data());

    return raw;
}

MeshNorms MeshBlob::norms() const noexcept
{
    MeshNorms raw;

    raw.count = static_cast<std::uint32_t>(m_d->norms.size());
    raw.ptr = reinterpret_cast<const float*>(m_d->norms.data());

    return raw;
}

MeshTris MeshBlob::tris() const noexcept
{
    MeshTris raw;

    raw.count = static_cast<std::uint32_t>(m_d->tris.size());
    raw.ptr = reinterpret_cast<const std::uint32_t*>(m_d->tris.data());

    return raw;
}

std::unique_ptr<MeshBlob> make_flask(double width, double thickness, double height) noexcept
{
    TopoDS_Shape body;

    // Extrude
    {
        gp_Pnt pts[5] = {
            {-width * 0.5, 0.0,               0.0},
            {-width * 0.5, 0.0, -thickness * 0.25},
            {         0.0, 0.0,  -thickness * 0.5},
            { width * 0.5, 0.0, -thickness * 0.25},
            { width * 0.5, 0.0,               0.0}
        };

        Handle(Geom_TrimmedCurve) arc = GC_MakeArcOfCircle(pts[1], pts[2], pts[3]);
        Handle(Geom_TrimmedCurve) seg1 = GC_MakeSegment(pts[0], pts[1]);
        Handle(Geom_TrimmedCurve) seg2 = GC_MakeSegment(pts[3], pts[4]);

        TopoDS_Wire wire1 = BRepBuilderAPI_MakeWire(BRepBuilderAPI_MakeEdge(seg1),
            BRepBuilderAPI_MakeEdge(arc),
            BRepBuilderAPI_MakeEdge(seg2));

        gp_Trsf xf;

        xf.SetMirror(gp::OX());

        TopoDS_Wire wire2 = TopoDS::Wire(BRepBuilderAPI_Transform(wire1, xf).Shape());

        BRepBuilderAPI_MakeWire mk_wire;

        mk_wire.Add(wire1);
        mk_wire.Add(wire2);

        body = BRepPrimAPI_MakePrism(BRepBuilderAPI_MakeFace(mk_wire.Wire()), gp_Vec(0.0, height, 0.0));
    }

    // Fillet cask
    {
        BRepFilletAPI_MakeFillet mk_fillet(body);

        for(TopExp_Explorer edge_i(body, TopAbs_EDGE); edge_i.More(); edge_i.Next())
        {
            mk_fillet.Add(thickness / 12.0, TopoDS::Edge(edge_i.Current()));
        }

        body = mk_fillet.Shape();
    }

    // Fuse with neck
    {
        body = BRepAlgoAPI_Fuse(body, BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(0.0, height, 0.0), gp::DY()), thickness / 4., height / 10.));
    }

    // Mesh it
    BRepTools::Clean(body);

    Handle(BRepMesh_DiscretRoot) discret_algo = BRepMesh_DiscretFactory::Get().Discret(body, 0.01, 12.0 * M_PI / 180.0);

    if(!discret_algo.IsNull())
    {
        discret_algo->Perform();
    }

    std::vector<Vertex> verts;
    std::vector<Vertex> norms;
    std::vector<Triangle> tris;

    verts.reserve(1024);
    norms.reserve(1024);
    tris.reserve(1024);

    TopLoc_Location loc;

    for(TopExp_Explorer face_i(body, TopAbs_FACE); face_i.More(); face_i.Next())
    {
        TopoDS_Face face = TopoDS::Face(face_i.Current());
        Handle(Poly_Triangulation) face_tri = BRep_Tool::Triangulation(face, loc);

        if(face_tri.IsNull())
        {
            continue;
        }

        const int triangles_n = face_tri->NbTriangles();

        if(triangles_n < 1)
        {
            continue;
        }

        const std::uint32_t vert_start = static_cast<std::uint32_t>(verts.size());

        if(!face_tri->HasNormals())
        {
            face_tri->ComputeNormals();
        }

        const int nodes_n = face_tri->NbNodes();

        if(loc.IsIdentity())
        {
            for(int i = 1; i <= nodes_n; ++i)
            {
                const gp_XYZ pos = face_tri->Node(i).XYZ();

                verts.push_back({static_cast<float>(pos.X()),
                    static_cast<float>(pos.Y()),
                    static_cast<float>(pos.Z())});

                gp_XYZ norm = face_tri->Normal(i).XYZ();

                if(face.Orientation() == TopAbs_REVERSED)
                {
                    norm.Reverse();
                }

                norms.push_back({static_cast<float>(norm.X()),
                    static_cast<float>(norm.Y()),
                    static_cast<float>(norm.Z())});
            }
        }
        else
        {
            const gp_Trsf& trsf = loc.Transformation();

            for(int i = 1; i <= nodes_n; ++i)
            {
                const gp_XYZ pos = face_tri->Node(i).Transformed(trsf).XYZ();

                verts.push_back({static_cast<float>(pos.X()),
                    static_cast<float>(pos.Y()),
                    static_cast<float>(pos.Z())});

                gp_XYZ norm = face_tri->Normal(i).Transformed(trsf).XYZ();

                if(face.Orientation() == TopAbs_REVERSED)
                {
                    norm.Reverse();
                }

                norms.push_back({static_cast<float>(norm.X()),
                    static_cast<float>(norm.Y()),
                    static_cast<float>(norm.Z())});
            }
        }

        for(int i = 1; i <= triangles_n; ++i)
        {
            Standard_Integer idx[3];

            face_tri->Triangle(i).Get(idx[0], idx[1], idx[2]);

            tris.push_back({vert_start + static_cast<std::uint32_t>(idx[0] - 1),
                vert_start + static_cast<std::uint32_t>(idx[1] - 1),
                vert_start + static_cast<std::uint32_t>(idx[2] - 1)});
        }
    }

    MeshBlobInit init;

    init.verts = std::move(verts);
    init.norms = std::move(norms);
    init.tris = std::move(tris);

    return std::make_unique<MeshBlob>(std::move(init));
}

} // namespace occt