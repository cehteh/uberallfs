
# Table of Contents

1.  [Overview](#orgd0f6077)
    1.  [The Idea](#orgfb93553)
    2.  [Features and Goals](#orgcbc5660)
        1.  [Caching and pinning objects](#org17ce620)
        2.  [Offline use](#org5b69cff)
        3.  [Strong security and anonymity available](#org87939f9)
        4.  [mixed in private data](#orgcbe16b6)
        5.  [redundant storage](#org6cebcf3)
        6.  [Garbage collection / Balancing](#org1c69d2a)
        7.  [Striped parallel downloads](#org0ad2611)
    3.  [Design Choices](#orgaa4646f)
2.  [Components](#orgba78ead)
    1.  [Object Store](#org806801a)
    2.  [Frontends](#orgcdec77a)
    3.  [Object Discovery](#org9cfbc76)
    4.  [Object Synchronization](#org4ee21f7)
    5.  [Access Control](#orgc99227a)
    6.  [Network / Sessions](#org965f4ea)
    7.  [Node Discovery](#orgc164494)
    8.  [Key Management](#org9590750)
    9.  [Distributed PKI](#orge249891)
3.  [Object Store](#bd6e60d2-31a6-46f8-87ec-173f395ef49b)
    1.  [Identifier Types](#org4e2ae15)
        1.  [Plans](#org762556d)
    2.  [Object Types](#org01437fd)
        1.  [tree](#org45de736)
        2.  [blob](#orgc4effff)
        3.  [part](#orgec5176e)
    3.  [Metadata Types](#org822ae0b)
        1.  [perm](#orga0c53f9)
        2.  [meta](#org4571861)
        3.  [dmap](#orgf33cacd)
        4.  [hash](#orga8f8573)
        5.  [link](#orgcacccab)
        6.  [rule](#org30274c9)
    4.  [Ideas](#org5f8e319)
4.  [Disk Layout](#org6cbbc3d)
    1.  [objectstore](#org17e2c4a)
    2.  [node](#org2932388)
    3.  [fuse](#org4a89f68)
5.  [Access Control](#62c4e059-5538-48a1-953a-43c1c9a5d7fb)
    1.  [Brainstorm/Ideas](#org3eec286)
    2.  [Security Implications](#orgb759839)
        1.  [replay attack](#org6b4d770)
        2.  [malicious object mutation](#orgd7e8943)
        3.  [privilege escalation](#org5a1a758)
        4.  [Object persistence](#org2332ef3)
    3.  [Concise Permissions](#org523d29b)
        1.  [File Permissions](#org3e19ea1)
        2.  [Directory Permissions](#org8cce863)
6.  [The Node](#d2f3ef15-6e9a-4cae-9131-1534664ffa98)
    1.  [Planned](#orge12fb2c)
        1.  [Realms](#orgce1d9c8)
7.  [HowTo](#ead96b87-abaf-43e6-89a8-111b9a8799d3)
    1.  [Plumbing vs Porcelain](#orgc13818d)
        1.  [Initialize and start a new uberallfs node](#org7f9170c)
8.  [Problems/Solutions](#org4b07c2b)
    1.  [Distributed object deletion](#orga8cc0d6)



<a id="orgd0f6077"></a>

# Overview


<a id="orgfb93553"></a>

## The Idea

Uberallfs is a peer to peer distributed filesystem. Objects are cached on the accessing
node. Mutation is implemented by passing Tokens around. Only the owner of the Token is
eligible to mutate the data. This token can be requested from the current token holder by any
entity which has permission to mutate the object. The old Token holder will then keep a
reference to the entity which received the token. These references can then be walked to
find the authoritative node at the end of the list. The actual implementation of this Idea
becomes a bit more complex and offers certain optimization opportunities.

Entities who don't have 'write' access do an object will never get the token. When read
access is requested they synchronize the object with the nodes who hold the original
data. This synchronization is managed by the current token owner.

[Look here for examples how to use it](#ead96b87-abaf-43e6-89a8-111b9a8799d3)


<a id="orgcbc5660"></a>

## Features and Goals


<a id="org17ce620"></a>

### Caching and pinning objects

Objects become cached upon access. There will be tools to enforce this caching and pin
objects to a node.


<a id="org5b69cff"></a>

### Offline use

The filesystem caches objects and can be used when a node is offline. For writes this
needs either the token to be locally obtained or the object can become 'detached' and
changes need to be merged when connectivity is restored (which can be automatic if there
are no conflicts).


<a id="org87939f9"></a>

### Strong security and anonymity available

Objects can be either instantiated by a 'creator' who defines security policies about who
has access to them OR published as anonymous/immutable.


<a id="orgcbe16b6"></a>

### mixed in private data

Objects may be private only and never be shared.


<a id="org6cebcf3"></a>

### redundant storage

Different levels of redundancy are planned, raid1 like redundancy where sync/close calls
only complete after the data is replicated or lazy backup schemes where data becomes
syncronized at lower priority without blocking current access.


<a id="org1c69d2a"></a>

### Garbage collection / Balancing

When space becomes scarce unused objects can be evicted from the cache. Either if this is
just a copy (that is not used for redundancy) by deleting the object or by offering
objects to other nodes within a configured realm of hosts.


<a id="org0ad2611"></a>

### Striped parallel downloads

If possible (later) Object transfer and syncronization can be spread over multiple peers
to utilize better bandwidth sharing (Bittorrent alike).


<a id="orgaa4646f"></a>

## Design Choices

Uberallfs uses 'opinionated' Design. Protocols include a single version number which fully
defines the properties, sizes and algorithms used. Future versions will be backward
compatible to few older versions but eventually old versions will become unsupported (which
may happen earlier when there are security related problems).

Version O is always defined to be 'experimental' it will be used in closed environments for
testing and development, never in production. Any Version 0 Protocol outside of this
environment is considered incompatible with itself.


<a id="orgba78ead"></a>

# Components

Following a coarse overview of the components making uberallfs. Details are described in
later Chapters.


<a id="org806801a"></a>

## Object Store

At the core is a object store where all filesystem objects are cached. Later support for
volatile objects is planned to allow once used streaming data. [For Details see below](#bd6e60d2-31a6-46f8-87ec-173f395ef49b).


<a id="orgcdec77a"></a>

## Frontends

User-access to the underlying filesystem hierarchy. The primary goal is a Linux fuse
filesystem which maps the underlying uberallfs to an ordinary POSIX conforming filesystem.

Later other front ends are planned. Android storage framework for example.


<a id="org9cfbc76"></a>

## Object Discovery

As described in the introduction, the 'trail' pointer used to locate the node which is
authoritative for a filesystem object is the main concept of uberallfs. Still there needs
to be more to make this functional. For example Objects need to be recovered when the trail
got broken (lost node). Only nodes which have full access to an object are allowed to
become authoritative.

When a node becomes authoritative this does not mean that the data is available there, it
only manages the 'ownership'. The object metadata contains references to nodes who
actually hold the data. For reading the data will be synchronized. While writing only
invalidates the old references and instantiates new data locally.

Nodes without full access to objects can synchronize data as far they have permissions to
do so and negotiate promises and leases with the authoritative node for race free data
access.


<a id="org4ee21f7"></a>

## Object Synchronization

Once access/authority to an object is granted the data may be synchronized (for reads).
For this maps of byte-ranges and version/generation counts are used. There is no need for
rsync like checksumming since the authoritative always knows which data is changed/recent.

Objects may become scattered across the nodes when frequent random writes at different
locations of an object happen. This is mitigated by a low priority object coalescing which
gather fragments and merges them on single nodes.


<a id="orgc99227a"></a>

## Access Control

Access control is implemented over public keys and signatures. The node which is
authoritative over an object is responsible for enforcing the permissions. Access control
metadata is sufficient enough to be freestanding without any additional information. Still
due to the distributed nature there are some loopholes that can not be closed (discussed
below). Basically any access ever granted can not be reliably revoked at a later time.

[Details below.](#62c4e059-5538-48a1-953a-43c1c9a5d7fb)


<a id="org965f4ea"></a>

## Network / Sessions

A node establishes a session with another node on behalf of a user/key. Each session is
then authenticated for this keys which is used for access control. Sessions are keep state
for some operations. As long a session is alive these states are valid. When a session dies
unexpectedly then these states and all associated data gets cleaned up/rolled back.

[Handled by the Node](#d2f3ef15-6e9a-4cae-9131-1534664ffa98).


<a id="orgc164494"></a>

## Node Discovery

Nodes are addressed by their public keys. The last seen addresses and names of other nodes
are cached for fast lookup. If that fails then a discovery is initiated (Details to be
worked out).


<a id="org9590750"></a>

## Key Management

creates user and node keys, manages signatures/pki,
key-agent process.


<a id="orge249891"></a>

## Distributed PKI

Future versions will include a distributed public key infrastructure. This augments the
exiting Access control with more advanced features like:

-   web of trust for confirming identity and credibility of other keys
-   revoking signatures
-   key aliasing/delegation
-   key renewal.


<a id="bd6e60d2-31a6-46f8-87ec-173f395ef49b"></a>

# Object Store

While uberallfs looks like a hierarchical filesystem, the backend store is a flat key/value
object store. The keys are derived from universally unique and secure identifiers. Secure in
this context means that not entity can create a collision that goes unnoticed. These
identifiers resemble global unique inode numbers.

There are different object types of objects stored under a key, explained later in this
document. The main parts are the 'tree' and 'blob' types. A 'tree' is an object that holds
named references to sub-object keys much like a directory in a filesystem. Blob objects
contain the file data. Other types contain metadata for security and distribution.

A mounted uberallfs uses a 'tree' object as the root of the mountpoint. From
there on a hierarchy like with any other filesystem is created.

The difference here is that all objects can be distributed over the network and anyone (with
permission to access the object) can references them within his own hierarchy. This for
example allows a complete home directory to be shared as well as mounting the same object
(directory) under different names at different positions in the hierarchy. For example one
instance may name a directory './Work/' and another one refers to the same tree object as
'./Arbeit/'.

Eventually (if one is careless) this could lead to directory cycles, which is the major
difference to traditional filesystems where directory cycles are highly disregarded.


<a id="org4e2ae15"></a>

## Identifier Types

A mutable objects are identified by a unique (random) number while an immutable object is
identified by a hash over its content. Objects which are constrained by permissions a
digital signature is required to guarantee integrity (see below).

We can further deduce the necessity of 3 scopes where these keys are valid:

1.  private objects that must never be shared but is accessible to the local instance
2.  public objects that have ownership and access permissions
3.  anonymous objects without any ownership and public access

This leads to following 4 types of identifiers:

<table border="2" cellspacing="0" cellpadding="6" rules="groups" frame="hsides">


<colgroup>
<col  class="org-left" />

<col  class="org-left" />

<col  class="org-left" />

<col  class="org-left" />
</colgroup>
<thead>
<tr>
<th scope="col" class="org-left">&#xa0;</th>
<th scope="col" class="org-left">private</th>
<th scope="col" class="org-left">public</th>
<th scope="col" class="org-left">anonymous</th>
</tr>
</thead>

<tbody>
<tr>
<td class="org-left">mutable</td>
<td class="org-left">random</td>
<td class="org-left">random signature</td>
<td class="org-left">¹</td>
</tr>


<tr>
<td class="org-left">immutable</td>
<td class="org-left">²</td>
<td class="org-left">hash signature</td>
<td class="org-left">hash</td>
</tr>
</tbody>
</table>

Note that there are 2 not supported combinations:

1.  Anonymous mutable data would lead security problems like denial of service attacks
2.  Having immutable private objects won't have any security implications and may be
    supported at some point when need arises (eg. deduplication)

Eventually some more Types might be supported, for example hashing could be indirect being
the hash over a bittorrent like list of hashes. This may even become the default for
immutable objects at some point.


<a id="org762556d"></a>

### Plans

Later file encryption might be added. This is not directly on topic for uberallfs as
objects are only distributed to nodes that are allowed to (at least) read them. File
encryption would remove this requirement and allow proxying/caching on nodes that which
don't have access to the object.


<a id="org01437fd"></a>

## Object Types

Details explained in the next chapter.


<a id="org45de736"></a>

### tree

Stores references to other objects (trees, blobs, symlinks) May store Unix special files
(fifo, sockets, device nodes) initially private, eventually network transparent nodes may
be implemented.


<a id="orgc4effff"></a>

### blob

The actual object (file) data.
can be sparse/incomplete with not yet synchronized data.


<a id="orgec5176e"></a>

### part

WIP: parts of blobs with own identifiers.


<a id="org822ae0b"></a>

## Metadata Types


<a id="orga0c53f9"></a>

### perm

Security manifest, access control and security related metadata.


<a id="org4571861"></a>

### meta

Extra metadata about authority/trail/generation/distribution.


<a id="orgf33cacd"></a>

### dmap

Maps to the nodes holding the data for mutable files. Initially only complete objects,
later byte ranges/multi node.


<a id="orga8f8573"></a>

### hash

Torrent like hash list for immutable files.


<a id="orgcacccab"></a>

### link

When an object type changes, its identifier changes. This .link type is then a pointer to
the new identifier.


<a id="org30274c9"></a>

### rule

-   Size restrictions for files.
-   Accepted filename patterns.
-   dirs/files only.
-   Change the properties/identifier of a file, eg. a when a '.mkv.part' file becomes
    renamed to '.mkv' its type is changed to 'public immutable'.

It is planned to make a simple rule engine that automates policies on objects (mostly
directories). For example:


<a id="org5f8e319"></a>

## Ideas

Keep lazy stats (coarse granularity, infrequently written to disk, with risk of loosing data in a crash)

-   **atime:** know when the object was last used
-   **afreq:** average frequency of use (rolling average?)


<a id="org6cbbc3d"></a>

# Disk Layout

There are (so far) three main components which need to be visible on the host
filesystem. These are designed to be in the same place (shared directory) as well as in
different places with the components shared over several uberallfs instances.

The basic use case is that all data resides in a single directory which also serves as
mountpoint for the fuse filesystem, thus shadowing they underlying data.


<a id="org17e2c4a"></a>

## objectstore

The objectstore can be freestanding/self contained no external configuration is needed.

-   **objects/:** used for the objectstore
-   **objects/??/:** any 2 character dir is used for the first level (4096 dirs, base64)
-   **objects/root/:** symlink to the root dir object
-   **objects/tmp/:** for safe tempfile handling
-   **objects/delete/:** deleted objects with some grace period
-   **objects/volatile:** can be a tmpfs for temporary objects
-   **objects/volatile/??/:** any 2 character dir is used for the first level (4096 dirs)
-   **config/:** configuration files
-   **objectstore.version:** version identifier

Planned: links to other objectstores on local computer, possibly on slower media for archives.


<a id="org2932388"></a>

## node

The 'node' manages the data distribution between other nodes, forming a peer to peer network.

For that it keeps the networks addresses of other nodes and manages network related keys.

-   **config/:** configuration files
-   **nodes/??/:** information about other nodes
-   **keystore/:** some of the keys used to operate the node. Others may be in ~/.config/uberallfs and are
    loaded on startup. Private keys will be isolated, TBD.
-   **uberallfs.sock:** socket for local node control
-   **node.version:** version identifier


<a id="org4a89f68"></a>

## fuse

When fuse gets mounted it may shadow all of the above and present POSIX compatible
file system.  Only files starting with '.uberallfs.' at the root are reserved (control
socket etc).


<a id="62c4e059-5538-48a1-953a-43c1c9a5d7fb"></a>

# Access Control

The 'perm' object type contains all metadata necessary for access control for the associated object. Any
node is obliged to validate access rights on queries.

-   **Identification:** We must ensure that an Object Key and Identifier belongs to the Object in question and
    all following security metadata needs to be derived from this in a provable way. All
    public keys can be constrained by an expire date.
    -   **Identifier:** A random number.
    -   **Creator:** Public key of the Creator/expiration of this object. Can be only once used key which is
        deleted after initialization of the metadata. The expiration date here becomes part of
        the identifier. Once passed the object becomes invalid and can be purged.
    -   **Identifier Signature:** The Identifier is signed with the Creators key.
    -   **Object Key:** The Identifier and its Signature are hashed together to give the key used in the
        object store. This is not stored in the 'perm' object as it is the 'name' thereof
        itself.

-   **Administrative Lists:** -   **Super Admins:** A (optional) list of public key/expire tupes that are allowed to modify the
        per-permission admins below.
        -   **Super Admins Signature:** The list of Super-Admins together with a nonce and the Identifier becomes signed by
            the Creator. This indirection allows to dispose the Creator key now and to delegate
            administrative task to multiple entities. Caveat: after the Creator key is disposed
            the Super-Admin list can not be changed anymore.
    
    -   **Per Permission Admins:** Optional list for each possible permission (read, write, delete, append, &#x2026;). Keys
        listed in these lists are allowed to modify the respective ACL's below. (idea:
        permission tags on the lists itself: an admin may add/delete&#x2026;)
        -   **Per Permission Admins Signature:** Each of the lists above needs to be signed by the Creator or a Super-Admin.
            This signature contains a nonce and the Identifier as well

-   **Access Control Lists:** Optional list for each possible permission (read, write, delete, append, &#x2026;). Keys
    listed in these lists are allowed to access the object in requested way.
    -   **ACL Signature:** Each of the lists above needs to be signed by the Creator or a Super-Admin or a
        matching per-permission-Admin. This signature contains a nonce and the Identifier as
        well.

-   **Generation Count and Signature:** Whenever any data on the above got changed a generation counter is incremented and the
    all list blocks plus this generation counter must be signed by one of the above
    administrative Keys (usually the one who did the change).

TODO: creation date and expire parameters are required, shall these be signed here?


<a id="org3eec286"></a>

## Brainstorm/Ideas

-   **Quorum:** M of N Admins must grant permission to be effective

-   **Key revocation:** special tree object which holds revoked signatures, must be safe
    against DoS, needs some thinking.


<a id="orgb759839"></a>

## Security Implications


<a id="org6b4d770"></a>

### replay attack

TBD: in short one who once had (administrative) access to the object can replay that old
version of the metadata under some conditions since the 'trail' and generation count can
be incomplete. (write example how this can happen, any solution for this?)

1.  A creates a file with B and C as Admin
2.  B takes the token from A   A->B
3.  C takes the token from B   A->B->C
4.  C removes B from an Administrative list
5.  B takes the token from C back  A->B<-C
6.  B replays the 'perm' metadata from 2. (gains Admin back)
7.  A takes the file from B but can not discover the tampering

The only 'weak' protection against this are the expiration dates. When these are short
enough they limit the time window in which such an attack can be done and constrain the
necessary lifetime for signature revocations.


<a id="orgd7e8943"></a>

### malicious object mutation

Can not happen because the token will never be given to a node that won't have write access.


<a id="org5a1a758"></a>

### privilege escalation


<a id="org2332ef3"></a>

### Object persistence


<a id="org523d29b"></a>

## Concise Permissions

Uberallfs implements a set of *concise permissions* unlike traditional 'rwx' Unix
permissions with their overloaded meaning for directories.

These permissions are mapped onto the available permissions of the target operating
system. Permissions are tied to (lists of) public keys. There are no users and groups
otherwise. There is one special (all zero?) Key which means 'anyone'.

A permission which would allow full access (including deleting/overwriting) all data also
allows a node to take authority over an object. Nodes which can't gain authority over an
object must pass their mutations to the authoritative node where they will be validated.

Access control is inclusive, when one could gain access because the key is listed in the
respective Admin list, then one gets that permission implicitly.


<a id="org3e19ea1"></a>

### File Permissions

File permission are initially relatively simple, only 'append' added over unix
permissions. Should be self explanatory.

-   **read:** 

-   **write:** This is the **authoritative** permission.
-   **append:** 


<a id="org8cce863"></a>

### Directory Permissions

**WIP!**

With directories things become more complicated.

-   **list:** Allow listing of the directory filenames.
    (purely know they exists, no object identifiers)
-   **list-accessible:** Listing is filtered to content where one has (any) access to.
-   **list-authoritative:** Listing is filtered to content where one has authority for.
-   **read:** Allow listing of the directory content including object identifiers.
-   **read-accessible:** Listing is filtered to content where one has (any) access to.
-   **read-authoritative:** Listing is filtered to content where one has authority for.
-   **add:** Add new objects.
-   **add-authoritative:** Only add objects where one is authoritative for.
-   **add-anonymous:** Add anonymous objects.
-   **rename:** Rename an object within the same directory. Moving objects across directories are
    handled like add/delete on each directory.
-   **rename-authoritative:** Rename an object within the same directory where one is authoritative for.
-   **rename-anonymous:** Rename an anonymous object within the same directory.
-   **delete:** Delete any object.
    This is the **authoritative** permission.
-   **delete-authoritative:** Delete objects where one is authoritative for.
-   **delete-anonymous:** Delete anonymous objects.

Further rules can be defined how objects are created, what extra permissions and keys apply (inherit from directory,..)


<a id="d2f3ef15-6e9a-4cae-9131-1534664ffa98"></a>

# The Node


<a id="orge12fb2c"></a>

## Planned


<a id="orgce1d9c8"></a>

### Realms


<a id="ead96b87-abaf-43e6-89a8-111b9a8799d3"></a>

# HowTo

WIP: Envisioned usage

Examples here using defaults for most options. Defaults should always be the be safe option.


<a id="orgc13818d"></a>

## Plumbing vs Porcelain

This examples starting with 'plumbing' commands to show the steps involved to set something
up. When applicable 'porcelain' is added next to it, in general porcelain commands simplify
usage, but depend on some preconditions, like that the filesystem is already set up and
mounted (unless for the setup commands).


<a id="org7f9170c"></a>

### Initialize and start a new uberallfs node

1.  With private root

        $ uberallfs objectstore ./DIR_A init
        $ uberallfs node ./DIR_A init
        $ uberallfs node ./DIR_A start
        $ uberallfs fuse ./DIR_A mount
    
        $ uberallfs init ./DIR_A
        $ uberallfs start ./DIR_A
    
    Will result in a uberallfs mounted on './DIR<sub>A</sub>' with a private (by default) root
    directory.

2.  Make a Directory shareable

    We created a 'private' root directory in the previous step. For being used as distributed
    directory its type must be changed.
    
        $ uberallfs objectstore ./DIR_A chtype public_mutable /
    
    This changes the type and sets up a minimal ACL to make the executing user Creator of the
    object.
    
    Porcelain will only work on a running (mounted) filesystem.
    
        $ uberallfs chtype public_mutable ./DIR_A

3.  Shared Root Dir

    The root directory is nothing special an can be shared as any other object, the only
    difference is that the root directory must be present in the objectstore for almost all
    other operations (like mounting the file system). Thus objectstore initialization can
    already takes care for setting up the root directory.
    
    On the new filesystem the node must be initialized first for exporting the (default
    generated) users public key.
    
        $ uberallfs node ./DIR_B init
        $ uberallfs node ./DIR_B export-key
        base64encodedpubkey
    
        $ uberallfs node ./DIR_B init
        $ uberallfs export_key ./DIR_B
        base64encodedpubkey
    
    -   By exported Directory
        
        Give the new user/key access to the root directory in './DIR<sub>A</sub>' and export it into an
        archive. This thin export only contains the minimum necessary metadata to reconstruct
        the content by querying the original node.
        
            $ uberallfs objectstore ./DIR_A chacl +super_admin base64encodedpubkey /
            $ uberallfs objectstore ./DIR_A send --thin / >ARCHIVE
        
            $ uberallfs chacl +super_admin base64encodedpubkey /DIR_A
            $ uberallfs export ./DIR_A ARCHIVE
        
        Now we can import that archive as new root directory and go on.
        
            $ uberallfs objectstore ./DIR_B init --import ARCHIVE
            $ uberallfs node ./DIR_B start
            $ uberallfs fuse ./DIR_B mount
        
            $ uberallfs import --root ARCHIVE ./DIR_B
            $ uberallfs start ./DIR_B
    
    -   By URL
        
        Instead importing an ARCHIVE one can also supply a URL the root dir will then be
        fetched over the network.
        
        The an URL has the form 'uberallfs://host:port/identifier' and can be shown by:
        
            $ uberallfs node ./DIR_A show --url /
            uberallfs://localhost:port/base64encodedidentifier
        
            $ uberallfs show-url ./DIR_A
            uberallfs://localhost:port/base64encodedidentifier
        
        This URL can then be used to bootstrap the new objectstore
        
            $ uberallfs objectstore ./DIR_B init --no-root
            $ uberallfs node ./DIR_B start
            $ uberallfs node ./DIR_B fetch uberallfs://localhost:port/base64encodedidentifier
            $ uberallfs objectstore ./DIR_B root --set base64encodedidentifier
            $ uberallfs fuse ./DIR_B mount
        
        'insta' does all DWIM magic to get a uberallfs running. initialization, starting and
        mounting the node. Possibly it asks some interactive questions (for deploying keys).
        An existing dir will be reused if no data gets overwritten (same root again).
        
            $ uberallfs insta ./DIR_B --from uberallfs://localhost:port/base64encodedidentifier


<a id="org4b07c2b"></a>

# Problems/Solutions


<a id="orga8cc0d6"></a>

## Distributed object deletion

Objects may be referenced from different locations all over the network. Deleting a object
from a directory is as simple as just remove it from there when one has authority over the
directory. But this does not mean the Object itself can be removed from the object store
since other nodes may still refer to it.

-   **Solutions:** -   When no parts of the object are locally authoritative (no data!) then it can be removed.
    -   Every Object has a 'grace' time for which it will be kept with a 'deleted' flag. Once
        this grace time is expired it can be deleted.
        
        -   Any other node which references this object should poll the object within this grace
            time. When the authoritative node responds that the object ought to be deleted then
            -   Node without full access are advised to synchronize the object
            -   Nodes with full access are advised to adopt the object.
                -   Once adopted and all data is transferred the **data** can deleted. Metadata (trail)
                    needs to stay alive until the grace time is expired.
        
        This grace time can be exponential, starting from for example 30 seconds, doubling on
        every expire where the object is still in use up to some upper limit.

