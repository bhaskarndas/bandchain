module Styles = {
  open Css;

  let barContainer =
    style([
      position(`relative),
      paddingTop(`px(20)),
      Media.mobile([display(`flex), alignItems(`center), paddingTop(`zero)]),
    ]);
  let progressOuter =
    style([
      position(`relative),
      width(`percent(100.)),
      height(`px(12)),
      borderRadius(`px(7)),
      border(`px(1), `solid, Colors.gray9),
      padding(`px(1)),
      overflow(`hidden),
    ]);
  let progressInner = (p, success) =>
    style([
      width(`percent(p)),
      height(`percent(100.)),
      borderRadius(`px(7)),
      background(success ? Colors.bandBlue : Colors.red4),
    ]);
  let leftText =
    style([
      position(`absolute),
      top(`zero),
      left(`zero),
      Media.mobile([
        position(`static),
        flexGrow(0.),
        flexShrink(0.),
        flexBasis(`px(50)),
        paddingRight(`px(10)),
      ]),
    ]);
  let rightText =
    style([
      position(`absolute),
      top(`zero),
      right(`zero),
      Media.mobile([
        position(`static),
        flexGrow(0.),
        flexShrink(0.),
        flexBasis(`px(70)),
        paddingLeft(`px(10)),
      ]),
    ]);
};

[@react.component]
let make = (~reportedValidators, ~minimumValidators, ~requestValidators) => {
  let progressPercentage =
    (reportedValidators * 100 |> float_of_int) /. (requestValidators |> float_of_int);
  let success = reportedValidators >= minimumValidators;

  <div className=Styles.barContainer>
    <div className=Styles.leftText>
      <Text value={"Min " ++ (minimumValidators |> Format.iPretty)} color=Colors.gray7 />
    </div>
    <div className=Styles.progressOuter>
      <div className={Styles.progressInner(progressPercentage, success)} />
    </div>
    <div className=Styles.rightText>
      <Text
        value={
          (reportedValidators |> Format.iPretty) ++ " of " ++ (requestValidators |> Format.iPretty)
        }
        color=Colors.gray7
      />
    </div>
  </div>;
};
