import React, { useEffect, useState } from 'react'
import { Statistic } from 'antd'
import { solanaToUSD } from '../../utils/assets'

interface IAmountLabel {
  amount: number,
  displayUSD?: boolean,
  title?: string,
}

export const AmountLabel = (props: IAmountLabel) => {
  const {amount, displayUSD = true, title = ""} = props

  const [USDamt, setUSDamt] = useState<number>(0)

  useEffect(() => {
    solanaToUSD(amount).then(setUSDamt)
  }, [amount])

  return <div style={{ display: 'flex' }}>
    <Statistic
      className="create-statistic"
      title={title}
      value={amount.toFixed(2)}
      prefix="◎"
    />
    {displayUSD &&
      <div
        style={{
          margin: 'auto 0',
          color: 'rgba(255, 255, 255, 0.4)',
          fontSize: '1.5rem',
        }}
      >
        ${USDamt.toFixed(2)}
      </div>
    }
  </div>
}
