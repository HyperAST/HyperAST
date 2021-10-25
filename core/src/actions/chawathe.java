package gumtree.spoon.apply.operations;

import com.github.gumtreediff.actions.EditScript;
import com.github.gumtreediff.actions.MyAction;
import com.github.gumtreediff.actions.VersionedEditScript;
import com.github.gumtreediff.actions.VersionedEditScriptGenerator;
import com.github.gumtreediff.actions.MyAction.MyDelete;
import com.github.gumtreediff.actions.model.*;
import com.github.gumtreediff.matchers.Mapping;
import com.github.gumtreediff.matchers.MappingStore;
import com.github.gumtreediff.matchers.Matcher;
import com.github.gumtreediff.matchers.MultiVersionMappingStore;
import com.github.gumtreediff.matchers.SingleVersionMappingStore;
import com.github.gumtreediff.tree.AbstractTree;
import com.github.gumtreediff.tree.AbstractVersionedTree;
import com.github.gumtreediff.tree.ITree;
import com.github.gumtreediff.tree.Tree;
import com.github.gumtreediff.tree.TreeUtils;
import com.github.gumtreediff.tree.Version;
import com.github.gumtreediff.tree.VersionedTree;

import gnu.trove.map.TIntObjectMap;
import gumtree.spoon.apply.MyUtils;
import gumtree.spoon.builder.SpoonGumTreeBuilder;
import spoon.reflect.declaration.CtElement;

import java.lang.reflect.Field;
import java.util.*;

public class MyScriptGenerator extends VersionedEditScriptGenerator {
    AbstractVersionedTree middle = null;

    public enum Granularity {
        ATOMIC, COMPOSE, SPLITED;
    }

    final Granularity granularity;
    Version beforeVersion;
    Version afterVersion;
    private Map<Version, Map<ITree, AbstractVersionedTree>> mappingPerVersion;

    public MyScriptGenerator(AbstractVersionedTree middle, Version initialVersion , Map<Version,Map<ITree,AbstractVersionedTree>> mappingPerVersion, Granularity granularity) {
        super(middle, initialVersion);
        this.middle = middle;
        this.granularity = granularity;
        this.mappingPerVersion = mappingPerVersion;
    }

    @Override
    public VersionedEditScript computeActionsForward(Matcher ms, Version beforeVersion, Version afterVersion) {
        this.beforeVersion = beforeVersion;
        this.afterVersion = afterVersion;
        initWith((SingleVersionMappingStore<AbstractVersionedTree, ITree>) ms.getMappings());
        generate();
        middle.setParent(null);
        origDst.setParent(null);
        return actions;
    }

    private ITree origSrc;

    private ITree origDst;

    private SingleVersionMappingStore<AbstractVersionedTree, ITree> origMappings;

    private SingleVersionMappingStore<AbstractVersionedTree, ITree> cpyMappings;

    private Set<ITree> dstInOrder;

    private Set<ITree> srcInOrder;

    // private EditScript actions;

    private Map<ITree, AbstractVersionedTree> origToCopy;

    private Map<AbstractVersionedTree, ITree> copyToOrig;
    private Map<ITree, MyDelete> deletesWaiting = new HashMap<>();

    public void initWith(SingleVersionMappingStore<AbstractVersionedTree, ITree> ms) {
        this.origMappings = ms;
        this.origSrc = ms.getSrc();
        this.origDst = ms.getDst();

        origToCopy = new HashMap<>();
        copyToOrig = new HashMap<>();
        cpyMappings = new SingleVersionMappingStore<AbstractVersionedTree, ITree>(middle, origDst);

        relateMiddleAndSource(middle, origSrc);

        for (Mapping m : origMappings) {
            cpyMappings.link(origToCopy.get(m.first), m.second);
            // multiVersionMappingStore.link(origToCopy.get(m.first), m.second);
        }
    }

    private void relateMiddleAndSource(AbstractVersionedTree cpyTree, ITree locOrigSrc) {
        origToCopy.put(locOrigSrc, cpyTree);
        // copyToOrig.put(cpyTree, origTree);
        List<AbstractVersionedTree> cpyChildren = (List) cpyTree.getChildren();
        List<ITree> origChildren = locOrigSrc.getChildren();
        if (cpyChildren.size() != origChildren.size()) {
            throw new RuntimeException("not same number of children");
        }
        for (int i = 0; i < cpyChildren.size(); i++) {
            relateMiddleAndSource(cpyChildren.get(i), origChildren.get(i));
        }
    }

    public EditScript generate() {
        AbstractVersionedTree srcFakeRoot = new AbstractVersionedTree.FakeTree(middle);
        ITree dstFakeRoot = new AbstractTree.FakeTree(origDst);
        middle.setParent(srcFakeRoot);
        origDst.setParent(dstFakeRoot);

        dstInOrder = new HashSet<>();
        srcInOrder = new HashSet<>();
        cpyMappings.link(srcFakeRoot, dstFakeRoot);

        // Set<ITree> deleted = new HashSet<>();
        // Set<ITree> added = new HashSet<>();
        ITree tree = origDst;
        handleInsMovUpdInit(tree.getChild(0), middle.getChild(null, 0));

        handleDeletion(middle);

        return actions;
    }

    private void handleInsMovUpdInit(ITree x, AbstractVersionedTree ww) {
        AbstractVersionedTree w = cpyMappings.getSrc(x);
        ITree y = x.getParent();
        AbstractVersionedTree z = cpyMappings.getSrc(y);
        if (w == null) { // no mapping
            List<AbstractVersionedTree> middleChildren = ww.getChildren(this.beforeVersion);
            List<ITree> origChildren = x.getChildren();
            if (ww.getType() != x.getType()) {
                handleInsMovUpd(x);
            } else if (!ww.getLabel().equals(x.getLabel())) {
                cpyMappings.link(ww, x);
                handleInsMovUpd(x);
            } else if (middleChildren.size() != origChildren.size()) {
                cpyMappings.link(ww, x);
                for (ITree child : origChildren) {
                    handleInsMovUpd(child);
                }
            } else {
                cpyMappings.link(ww, x);
                for (int i = 0; i < middleChildren.size(); i++) {
                    handleInsMovUpdInit(origChildren.get(i), middleChildren.get(i));
                }
            }
        } else {
            handleInsMovUpd(x);
        }
    }

    private void handleInsMovUpd2(ITree x) {
        AbstractVersionedTree w = cpyMappings.getSrc(x);
        ITree y = x.getParent();//Tree
        AbstractVersionedTree z = cpyMappings.getSrc(y);
        boolean already = alreadyAna.contains(x);
        if (already) {
        } else if (w == null) { // no mapping

        }

        for (ITree child : x.getChildren()) {
            handleInsMovUpd2(child);
        }
    }

    private void handleInsMovUpd(ITree tree) {
        List<ITree> bfsDst = TreeUtils.breadthFirst(tree);
        for (ITree x : bfsDst) {
            AbstractVersionedTree w;
            ITree y = x.getParent();//Tree
            AbstractVersionedTree z = cpyMappings.getSrc(y);
            boolean already = alreadyAna.contains(x);
            if (already) {
                // continue; // not sure if I hould skip
            }
            if (!cpyMappings.hasDst(x)) {
                int k = findPos2(x, y);//y.getChildPosition(x);
                // Insertion case : insert new node.
                w = new VersionedTree(x, this.afterVersion);
                // copyToOrig.put(w, x);
                cpyMappings.link(w, x);
                w.setParent(z);
                z.insertChild(w, k);
                Action action = addInsert(x, w);
                addInsertAction(action, w);
                mdForMiddle(x, w, mappingPerVersion.get(this.afterVersion));
            } else {
                w = cpyMappings.getSrc(x);
                if (!x.equals(origDst)) { // TODO => x != origDst // Case of the root
                    AbstractVersionedTree v = w.getParent();
                    if (!w.getLabel().equals(x.getLabel()) && !z.equals(v)) {
                        // x was renamed and moved from z to y
                        // in intermediate: w is moved from v to z,
                        // thus w is marked as deleted and newTree is created

                        int k = findPos2(x, y);//y.getChildPosition(x);
                        AbstractVersionedTree newTree = new VersionedTree(x, this.afterVersion);
                        newTree.setLabel(x.getLabel());
                        newTree.setParent(z);
                        z.insertChild(newTree, k);
                        cpyMappings.link(newTree, x);
                        // added.add(newTree);
                        // deleted.add(w);
                        // // copyToOrig.put(w, x);
                        // copyToOrig.put(newTree, x);
                        mdForMiddle(x, newTree, mappingPerVersion.get(this.afterVersion));

                        // Action uact = new MyAction.MyInsert(Update.class, w, wbis);
                        // addDeleteAction(uact, w);
                        // addInsertAction(uact, wbis);
                        newTree.setMetadata("alsoUpdated", true);
                        switch (granularity) {
                            case COMPOSE: {
                                // Move mact = null;//addMove(w, newTree); // TODO
                                // // add(mact);
                                // // add(uact);
                                // addDeleteAction(mact, w);
                                // addInsertAction(mact, newTree);
                                break;
                            }
                            case ATOMIC: {
                                Action iact = addInsert(w, newTree);
                                Action dact = addDelete(w);
                                w.delete(this.afterVersion);
                                addDeleteAction(dact, w);
                                addDeleteAction(dact, x);
                                addInsertAction(iact, newTree);
                                break;
                            }
                            case SPLITED: {
                                MyAction.MyMove mact = addMove(w, newTree);
                                Action iact = mact.getInsert();
                                MyDelete dact = mact.getDelete();
                                deletesWaiting.put(w, dact);
                                addInsertAction(iact, newTree);
                                addMoveAction(mact, x, w, newTree);
                                break;
                            }
                        }
                    } else if (!w.getLabel().equals(x.getLabel())) {
                        // x was renamed
                        // in intermediate: w is marked as deleted,
                        // newTree is created with new label
                        AbstractVersionedTree newTree = new VersionedTree(x, this.afterVersion);
                        cpyMappings.link(newTree, x);
                        // added.add(newTree);
                        // deleted.add(w);
                        int k = findPos2(w, v);
                        w.delete(this.afterVersion);
                        // cpyMappings.link(newTree, x);
                        // // copyToOrig.put(w, x);
                        // copyToOrig.put(newTree, x);
                        newTree.setParent(v);
                        v.insertChild(newTree, k);
                        newTree.setLabel(x.getLabel());
                        // mdForMiddle(x.getParent(), newTree.getParent());
                        Action action = addUpdate(w, newTree);
                        addDeleteAction(action, w);
                        addInsertAction(action, newTree);
                        mdForMiddle(x, newTree, mappingPerVersion.get(this.afterVersion));
                    } else if (!z.equals(v)) {
                        // x was moved from z to y
                        // in intermediate: w is was moved from v to z,
                        // thus w is marked as deleted and newTree is created
                        int k = findPos2(x, y);
                        AbstractVersionedTree newTree = new VersionedTree(x, this.afterVersion);
                        newTree.setParent(z);
                        z.insertChild(newTree, k);
                        cpyMappings.link(newTree, x);
                        // added.add(newTree);
                        // deleted.add(w);
                        // // copyToOrig.put(w, x);
                        // copyToOrig.put(newTree, x);
                        mdForMiddle(x, newTree, mappingPerVersion.get(this.afterVersion));
                        switch (granularity) {
                            case COMPOSE: {
                                // Action mact = new MyAction.MyMove(w, newTree);
                                // add(mact);
                                // addDeleteAction(mact, w);
                                // addDeleteAction(mact, x);
                                // addInsertAction(mact, newTree);
                                break;
                            }
                            case ATOMIC: {
                                Action iact = addInsert(w, newTree);
                                Action dact = addDelete(w);
                                w.delete(this.afterVersion);
                                addDeleteAction(dact, w);
                                addDeleteAction(dact, x);
                                addInsertAction(iact, newTree);
                                break;
                            }
                            case SPLITED: {
                                MyAction.MyMove mact = addMove(w, newTree);
                                Action iact = mact.getInsert();
                                MyDelete dact = mact.getDelete();
                                deletesWaiting.put(w, dact);
                                // w.delete(this.afterVersion);
                                // addDeleteAction(dact, w);
                                addInsertAction(iact, newTree);
                                addMoveAction(mact, x, w, newTree);
                                break;
                            }
                        }
                    } else {
                        mdForMiddle(x, w, mappingPerVersion.get(this.afterVersion));
                    }

                }
            }
            srcInOrder.add(w);
            dstInOrder.add(x);
            alignChildren(w, x);
        }
    }

    private int findPos2(ITree x, ITree y) {
        return y.getChildPosition(x);
        // return Math.max(0, y.getChildPosition(x) - (y.getChild(0).getMetadata("type").equals("LABEL") ? 1 : 0));
    }

    public static String ORIGINAL_SPOON_OBJECT_PER_VERSION = "ORIGINAL_SPOON_OBJECT_PER_VERSION";

    private void mdForMiddle(ITree original, AbstractVersionedTree middle, Map<ITree,AbstractVersionedTree> mapping) {
        CtElement ele = (CtElement) original.getMetadata(SpoonGumTreeBuilder.SPOON_OBJECT);
        if (ele == null) {
            ele = (CtElement) original.getParent().getMetadata(SpoonGumTreeBuilder.SPOON_OBJECT);
            middle = middle.getParent();
        } else {
            mapping.put(original, middle);
            // ele.putMetadata(VersionedTree.MIDDLE_GUMTREE_NODE, middle); // might cause mem leak
        }
        if (ele == null || middle == null) {
            return;
        }
        Map<Version, CtElement> tmp = (Map<Version, CtElement>) middle.getMetadata(ORIGINAL_SPOON_OBJECT_PER_VERSION);
        if (tmp == null) {
            tmp = new HashMap<>();
            middle.setMetadata(ORIGINAL_SPOON_OBJECT_PER_VERSION, tmp);
        }
        CtElement oldOri = (CtElement) middle.getMetadata(VersionedTree.ORIGINAL_SPOON_OBJECT);
        if (oldOri != null && !tmp.containsKey(this.beforeVersion)) {
            tmp.put(this.beforeVersion, oldOri);
        } else if (oldOri == null) {
            middle.setMetadata(VersionedTree.ORIGINAL_SPOON_OBJECT, ele);
        }
        tmp.put(this.afterVersion, ele);
    }

    private void handleDeletion(AbstractVersionedTree w) {
        List<AbstractVersionedTree> children = w.getAllChildren();
        for (AbstractVersionedTree child : children) {
            if (child.existsAt(this.beforeVersion)) {
                handleDeletion(child);
            }
        }
        if (deletesWaiting.containsKey(w)) {
            MyDelete dact = deletesWaiting.get(w);
            w.delete(this.afterVersion);
            add(dact);
            addDeleteAction(dact, w);
        } else if (!cpyMappings.hasSrc(w)) {
            if (w.getInsertVersion() == this.afterVersion) {
                System.err.println(w);
            } else {
                w.delete(this.afterVersion);
                Action action = addDelete(w);
                addDeleteAction(action, w);
            }
        }

        for (AbstractVersionedTree child : children) {
            if (child.existsAt(this.beforeVersion)) {
                actions.compressAtomic(child);
            } else if (child.existsAt(this.afterVersion)) {
                handleCompression(child);
            }
        }
    }

    private void handleCompression(AbstractVersionedTree w) {
        List<AbstractVersionedTree> children = w.getChildren(this.afterVersion);
        for (AbstractVersionedTree child : children) {
            handleCompression(child);
        }
        actions.compressAtomic(w);
    }

    public static String DELETE_ACTION = "DELETE_ACTION";
    public static String INSERT_ACTION = "INSERT_ACTION";

    private Action addDeleteAction(Action action, ITree w) {
        return (Action) w.setMetadata(DELETE_ACTION, action);
    }

    private Action addInsertAction(Action action, ITree w) {
        return (Action) w.setMetadata(INSERT_ACTION, action);
    }

    public static String MOVE_SRC_ACTION = "MOVE_SRC_ACTION";
    public static String MOVE_DST_ACTION = "MOVE_DST_ACTION";

    private void addMoveAction(Move action, ITree x, AbstractVersionedTree w, AbstractVersionedTree wbis) {
        x.setMetadata(MOVE_SRC_ACTION, action);
        assert w.setMetadata(MOVE_SRC_ACTION, action) == null;
        assert wbis.setMetadata(MOVE_DST_ACTION, action) == null;
    }

    Set<ITree> alreadyAna = new HashSet<>();

    private void alignChildren(ITree w, ITree x) {
        srcInOrder.removeAll(w.getChildren()); // TODO look at it !
        dstInOrder.removeAll(x.getChildren());

        List<ITree> s1 = new ArrayList<>();
        for (ITree c : w.getChildren())
            if (cpyMappings.hasSrc(c))
                if (x.getChildren().contains(cpyMappings.getDst(c)))
                    s1.add(c);

        List<ITree> s2 = new ArrayList<>();
        for (ITree c : x.getChildren())
            if (cpyMappings.hasDst(c))
                if (w.getChildren().contains(cpyMappings.getSrc(c)))
                    s2.add(c);

        List<Mapping> lcs = lcs(s1, s2);

        for (Mapping m : lcs) {
            srcInOrder.add(m.first);
            dstInOrder.add(m.second);
        }

        for (ITree b : s2) { // iterate through s2 first, to ensure left-to-right insertions
            for (ITree a : s1) {
                if (oriMappings.has(a, b)) {
                    if (!lcs.contains(new Mapping(a, b))) {
                        int k = findPos2(b, x);//x.getChildPosition(b);
                        AbstractVersionedTree newTree = new VersionedTree(b, this.afterVersion);
                        newTree.setParent(w);
                        w.insertChild(newTree, Math.min(w.getChildren().size(),k));
                        cpyMappings.link(newTree, b);
                        // // copyToOrig.put((AbstractVersionedTree) a, x);
                        // copyToOrig.put(newTree, x);
                        alreadyAna.add(b);
                        mdForMiddle(b, newTree, mappingPerVersion.get(this.afterVersion));
                        switch (granularity) {
                            case COMPOSE: {
                                // Action mact = new MyAction.MyMove(w, newTree);
                                // add(mact);
                                // addDeleteAction(mact, w);
                                // addDeleteAction(mact, x);
                                // addInsertAction(mact, newTree);
                                break;
                            }
                            case ATOMIC: {
                                Action iact = addInsert(a, newTree);
                                Action dact = addDelete((AbstractVersionedTree) a);
                                ((AbstractVersionedTree) a).delete(this.afterVersion);
                                addDeleteAction(dact, a);
                                addDeleteAction(dact, x);
                                addInsertAction(iact, newTree);
                                break;
                            }
                            case SPLITED: {
                                MyAction.MyMove mact = addMove((AbstractVersionedTree) a, newTree);
                                Action iact = mact.getInsert();
                                MyDelete dact = mact.getDelete();
                                deletesWaiting.put(a, dact);
                                // ((AbstractVersionedTree) a).delete(this.afterVersion);
                                // addDeleteAction(dact, a);
                                addInsertAction(iact, newTree);
                                addMoveAction(mact, b, (AbstractVersionedTree) a, newTree);
                                break;
                            }
                        }
                        srcInOrder.add(a);
                        dstInOrder.add(b);
                    }
                }
            }
        }
    }

    private int findPos(ITree x) {
        ITree y = x.getParent();
        List<ITree> siblings = y.getChildren();

        for (ITree c : siblings) {
            if (dstInOrder.contains(c)) {
                if (c.equals(x))
                    return 0;
                else
                    break;
            }
        }

        int xpos = x.positionInParent();
        ITree v = null;
        for (int i = 0; i < xpos; i++) {
            ITree c = siblings.get(i);
            if (dstInOrder.contains(c))
                v = c;
        }

        //if (v == null) throw new RuntimeException("No rightmost sibling in order");
        if (v == null)
            return 0;

        ITree u = cpyMappings.getSrc(v);
        // siblings = u.getParent().getChildren();
        // int upos = siblings.indexOf(u);
        int upos = u.positionInParent();
        // int r = 0;
        // for (int i = 0; i <= upos; i++)
        // if (srcInOrder.contains(siblings.get(i))) r++;
        return upos + 1;
    }

    private List<Mapping> lcs(List<ITree> x, List<ITree> y) {
        int m = x.size();
        int n = y.size();
        List<Mapping> lcs = new ArrayList<>();

        int[][] opt = new int[m + 1][n + 1];
        for (int i = m - 1; i >= 0; i--) {
            for (int j = n - 1; j >= 0; j--) {
                if (cpyMappings.getSrc(y.get(j)).equals(x.get(i)))
                    opt[i][j] = opt[i + 1][j + 1] + 1;
                else
                    opt[i][j] = Math.max(opt[i + 1][j], opt[i][j + 1]);
            }
        }

        int i = 0, j = 0;
        while (i < m && j < n) {
            if (cpyMappings.getSrc(y.get(j)).equals(x.get(i))) {
                lcs.add(new Mapping(x.get(i), y.get(j)));
                i++;
                j++;
            } else if (opt[i + 1][j] >= opt[i][j + 1])
                i++;
            else
                j++;
        }

        return lcs;
    }

    @Override
    public VersionedEditScript computeActionsBackward(Matcher matcher, Version beforeVersion, Version afterVersion) {
        // TODO Auto-generated method stub
        return null;
    }

    @Override
    public VersionedEditScript computeActionsSuplementary(Matcher matcher, Version beforeVersion,
            Version afterVersion) {
        // TODO Auto-generated method stub
        return null;
    }
}