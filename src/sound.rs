use std::collections::{HashMap, HashSet};

use crate::{force_sound_skip, SOKU_FRAMECOUNT};

pub struct RollbackSoundManager {
    sounds_that_did_happen: HashMap<usize, Vec<usize>>,
    sounds_that_maybe_happened: HashMap<usize, Vec<usize>>,
    pub current_rollback: Option<usize>,
}

impl RollbackSoundManager {

    // Create a new RollbackSoundManager and return
    pub fn new() -> Self {
        Self {
            sounds_that_did_happen: HashMap::new(),
            sounds_that_maybe_happened: HashMap::new(),
            current_rollback: None,
        }
    }

    pub fn insert_sound(&mut self, frame: usize, sound: usize) -> bool {
        //if let Some(x) = self.sounds_that_maybe_happened.get_mut(&frame) {
        //    let index = x.iter().position(|x| *x == sound);
        //    if let Some(idx) = index {
        //        //x.remove(idx);
        //        return false;
        //    }
        //}

        // 從 self內取得 entry = frame的Vec，也就是在 frame那個時候需要發生的 sound
        // 如果沒有這個元素，則新增一個並回傳給 s，然後再將 soundid 給 push進去。
        // sounds_that_did_happen 會記錄在哪個 frame 將會發生哪些 sound。
        let s = self
            .sounds_that_did_happen
            .entry(frame)
            .or_insert_with(|| Vec::new());
        s.push(sound);

        // 從sounds_that_maybe_happened，取出所有值，然後鋪平(變成一維陣列)
        // 然後從中尋找特定的 soundid。
        self.sounds_that_maybe_happened
            .values()
            .map(|x| x.iter())
            .flatten()
            .find(|x| **x == sound)
            .is_none()
    }

    // 從 from frame 到 to frame 內，將 sounds_that_did_happen的元素刪除，
    // 並重新加入回到 sounds_that_maybe_happened。
    pub fn pop_sounds_since(&mut self, from: usize, to: usize) {
        //println!("popping sounds from {from} to {to}");

        // Replace 用來交換 src 以及 dest 的 ownership，也就是兩個人會交換內容。
        // 不太確定這在幹嘛??????
        match std::mem::replace(&mut self.current_rollback, Some(from)) {
            Some(x) => println!("should have crashed sound 45"),
            None => (),
        };

        // 從sounds_that_did_happen，逐個 remove，
        // 然後將再重新新增進　sounds_that_maybe_happened。
        for a in from..=to {
            if let Some(old_sounds) = self.sounds_that_did_happen.remove(&a) {
                self.sounds_that_maybe_happened.insert(a, old_sounds);
            }
        }
    }

    pub fn delete_non_matched(&mut self) {

        // 先取得　sounds_that_maybe_happened 的 ownership
        let old_sounds = std::mem::replace(&mut self.sounds_that_maybe_happened, HashMap::new());
        let fc = unsafe { *SOKU_FRAMECOUNT };
        
        // current_rollback 指的可能是 frame，有 frame就回傳，沒有就報錯。
        let old = match self.current_rollback.take() {
            Some(x) => x,
            None => {
                println!("should have crashed here, sound 65");
                return;
                //panic!();
            }
        };

        // 從 sounds_that_did_happend 將所有元素取出，並且放進 new_sounds.
        let new_sounds: HashSet<usize> = (old..=fc)
            .flat_map(|x| self.sounds_that_did_happen.remove(&x))
            .map(|x| x.into_iter())
            .flatten()
            .collect();

        // 從 sounds_that_maybe_happened 內，將所有元素，依據 (frame, soundid)的方式，回傳
        // 所以整體會變成 for(frame, sound) in array<frame, soundid>
        // 會根據 frame排列也就是，第一個 frame以及他的所有 soundid，
        // 再來才是 第二個 frame 以及對應的 soundid。
        for (frame, sound) in old_sounds
            .into_iter()
            .map(|(frame, sounds)| sounds.into_iter().map(move |x| (frame, x)))
            .flatten()
        {
            //to do: not only delete sounds, but also restart them/shift them

            // 如果 sounds_that_did_happend 沒有 soundid 的紀錄
            if !new_sounds.contains(&sound) {

                // 檢查當前的 soundid 是否存在於最近的 5個 frame 中???
                // 我不知道為什麼要 - 5
                let played_recently = (old.saturating_sub(5)..old)
                    .filter_map(|x| self.sounds_that_did_happen.get(&x))
                    .map(|x| x.iter())
                    .flatten()
                    .find(|x| **x == sound)
                    .is_some();

                if !played_recently {
                    force_sound_skip(sound);

                    /*println!(
                        "sound {}, from frame {} deleted at frame {}",
                        sound,
                        frame,
                        unsafe { *SOKU_FRAMECOUNT },
                    ); */
                } else {
                    //println!("sound {} would have been skipped but wasnt", sound)
                }
            } else {
                self.sounds_that_did_happen
                    .entry(frame)
                    .or_insert_with(|| Vec::new())
                    .push(sound)
                //self.insert_sound(*frame, *sound);
                //println!("sound retained");
            }
        }
    }
}
