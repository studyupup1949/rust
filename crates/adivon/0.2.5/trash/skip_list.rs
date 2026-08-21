        if self.head.is_none() {
            self.head = Some(Box::new(SkipNode::new(key, it, self.level)));
            return;
        }

        let mut x: Rawlink<SkipNode<Key,E>> = self.head.as_mut().map(|h| Rawlink::some(&mut **h)).unwrap();

        let mut update: Vec<Rawlink<SkipNode<Key,E>>> = iter::repeat(Rawlink::none()).take(new_level).collect();
        for i in (0..new_level).rev() {
            let mut xn = x.resolve().map(|n| n.forward[i].resolve_mut().map(|n| n.to_ptr())).unwrap();
            while xn.unwrap().resolve().map_or(false, |n| n.key < key) {
                x = xn.unwrap();
                xn = x.resolve().map(|n| n.forward[i].resolve_mut().unwrap().to_ptr());
            }
            update[i] = x;
        }

        // level 0
        {
            let mut xn = x.resolve().map(|n| n.next.as_mut().map(|nx| nx.to_ptr())).unwrap().unwrap();
            while xn.unwrap().resolve().map_or(false, |n| n.key < key) {
                x = xn;
                xn = x.as_ref().map(|n| n.next.as_mut().map(|nx| &mut **nx)).unwrap();
            }
        }

        let nx = SkipNode::new(key, it, new_level);
        x.as_mut().map(|n| {
            n.next = Some(Box::new(nx));
        });
        let nx_p = x.as_mut().map_or_else(Rawlink::none, |n| {
            n.next.map(|x| Rawlink::some(&mut *x)).unwrap()
        });

        for lv in 0 .. new_level {
            x.as_mut().map(|n| n.forward[lv] = update[lv]);
            update[lv].resolve_mut().map(|n| n.forward[lv] = nx_p);
        }
        self.size += 1;
