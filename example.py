import numpy as np
import pandas as pd

from woebin import WoeBinningProc


np.random.seed(0)


def create_dataset(size, cat_num, is_sorted=False):
    # Probability of target for each category
    prob_map = np.random.random(cat_num)

    if is_sorted:
        prob_map.sort()

    # Probabilities for each category in dataset
    cat_probs = np.random.random(cat_num)
    cat_probs /= cat_probs.sum()

    # Category series according to cat_probs
    cat = np.random.choice(list(range(cat_num)), size=size, p=cat_probs)

    # Target series according to the probability for each category
    trg = (
        np.random.random(size) < np.vectorize(prob_map.__getitem__)(cat)
    ).astype(int)

    # Final dataset as pandas dataframe
    return pd.DataFrame({'series': cat, 'target': trg})


def example():
    size = 1_000_000
    desirable_bins_num = 5

    df = create_dataset(size, 1000, is_sorted=True)
    print(df)

    print("Process categorial")

    wbp = WoeBinningProc(desirable_bins_num)

    wbp.process_categorial(df['series'], df['target'])

    # bins_info = wbp.get_bins_info()
    # print(bins_info)

    # woe_map = wbp.get_woe_map()
    # print(woe_map)

    print("IV:", wbp.get_iv_total())

    print("Process numeric")

    wbp = WoeBinningProc(desirable_bins_num)

    wbp.process_numeric(df['series'], df['target'])

    # bins_info = wbp.get_bins_info()
    # print(bins_info)

    # woe_map = wbp.get_woe_map()
    # print(woe_map)

    print("IV:", wbp.get_iv_total())

    print("Completed")


if __name__ == "__main__":
    example()
