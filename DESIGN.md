# Design

Lets ditch dynamic state, just a 1024x1024 screen. Fixed size.


## Version One

We simply render shit on the CPU


## Primary rays
Primary rays have coherence. Shadow rays too.

They intersect the same primitives and tend to traverse the same BVH nodes.

Ray tracing is dominated by memory latency.

ammortize cost of fetching data over multiple rays.

```
BVHNode::Traverse(Ray r) {
  if(!intersectrs(bounds) return)
    if (siLeaf()) {
      IntersectPrimitives
    }
    else {
      pool[left].Traverse(r)
      pool[left+1].Traverse(r)
    }
}
```

vs.

IF ANY ray intersects, we traverse
```
BVHNode::Traverse(Ray4 r) {
  if(!intersectrs(bounds) return)
    if (siLeaf()) {
      IntersectPrimitives
    }
    else {
      pool[left].Traverse(r)
      pool[left+1].Traverse(r)
    }
}
```

More memory coherence! Even if we do not SIMD!

We do need to `mask` rays. A ray may become inactive.


Plane equation: 
           

N + d = 0
